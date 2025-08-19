from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    BitsAndBytesConfig,
    StoppingCriteriaList,
    StoppingCriteria,
)
import torch
import random
import re
import os
import subprocess

def extract_first_code_block(text):
    pattern = r'```(?:\w+)?\n?(.*?)```'
    match = re.search(pattern, text, re.DOTALL)
    if match:
        return match.group(1).strip()
    return ""

def gen_model():
    model_name = "Qwen/Qwen2.5-Coder-7B-Instruct"
    quantization_config = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_compute_dtype=torch.float16,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
    )
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForCausalLM.from_pretrained(
        model_name,
        quantization_config=quantization_config,
        device_map="auto",
        trust_remote_code=True,
    )
    print(f"Model loaded: {torch.cuda.memory_allocated()/1024**3:.2f}GB")
    return model, tokenizer

def gen_prompt(tokenizer, model_device, user_prompt):
    messages = [
        {"role": "system", "content": "You are a helpful coding assistant"},
        {"role": "user", "content": user_prompt}
    ]
    text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
    return tokenizer([text], return_tensors="pt").to(model_device)

def execute(model, tokenizer, input_prompt):
    with torch.no_grad():
        generated_ids = model.generate(
            **input_prompt,
            max_new_tokens=256,
            do_sample=True,
            repetition_penalty=1.0,
            temperature=1.0,
            top_p=0.7,
            pad_token_id=tokenizer.eos_token_id,
        )
        generated_ids = [output_ids[len(input_ids):] for input_ids, output_ids in zip(input_prompt.input_ids, generated_ids)]
        response = tokenizer.batch_decode(generated_ids, skip_special_tokens=True)[0]
        return response

def is_valid_code(c_code):
    result = subprocess.run(
        ['clang', '-x', 'c', '-'],
        input=c_code,
        text=True,
        capture_output=True
    )
    if result.returncode == 0:
        os.remove("./a.out")
    return result.returncode == 0

def gen_example(n_samples=5):
    model, tokenizer = gen_model()
    guide_prompt = '''
        Requirements:
        - Do not use any file-related functionality.
        - Only use standard library functions: malloc, free, and strcpy.
        - Use the function signature: void main().
        - Add a brief comment next to the buggy line indicating the type of bug.
        - Provide only one function, with no additional explanation or output.
        - **Include only one bug type per generated code.**
        '''
    newgen_bof_prompt = "Please create a program that triggers buffer overflow bug."
    newgen_uaf_prompt = "Please create a program that triggers use-after-free bug."
    se_prompt = "Please create a semantically equivalent program to the previous generation"
    mutate_prompt = "Please create a mutated program that modifies the previous generation"
    gen_stats = [newgen_bof_prompt, newgen_uaf_prompt, mutate_prompt, se_prompt]

    # generated mutated fuzzing inputs
    fuzzing_inputs = []
    while len(fuzzing_inputs) < n_samples:
        instruction = random.choice(gen_stats)
        print(f"generating ...{len(fuzzing_inputs)}")
        print(f"selected instruction: {instruction}")
        prompt_id = gen_prompt(tokenizer, model.device, guide_prompt + "\n" + instruction)
        fuzzing_input = execute(model, tokenizer, prompt_id)
        fuzzing_input = extract_first_code_block(fuzzing_input)
        if is_valid_code(fuzzing_input):
            fuzzing_inputs.append(fuzzing_input)
        else:
            print(f"compile failed: \n {fuzzing_input}")
    return fuzzing_inputs

def write_c_files(fuzzing_inputs):
   os.makedirs("generated", exist_ok=True)
   for i, code in enumerate(fuzzing_inputs):
       filename = f"generated/{i + 1}.c"
       with open(filename, 'w') as f:
           f.write(code)

n_samples = 5
examples = gen_example(n_samples)
write_c_files(examples)
