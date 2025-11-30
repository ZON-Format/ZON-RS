╔════════════════════════════════════════════════════════════════════════════╗
║                 ZON vs TOON vs CSV vs JSON BENCHMARK                       ║
║                   Token Efficiency Comparison                              ║
║                   Using GPT-5 o200k_base,Claude 3.5 (Anthropic),           ║
║                   Llama 3 (Meta) tokenizer                                 ║
╚════════════════════════════════════════════════════════════════════════════╝

════════════════════════════════════════════════════════════════════════════════
📊 Unified Dataset
   Combined dataset with tabular, nested, and time-series data
────────────────────────────────────────────────────────────────────────────────
📦 BYTE SIZES:
  ZON:              1,399 bytes
  TOON:             1,665 bytes
  CSV:              1,384 bytes
  YAML:             2,033 bytes
  XML:              3,235 bytes
  JSON (formatted): 2,842 bytes
  JSON (compact):   1,854 bytes

🔹 Tokenizer: GPT-4o (o200k)
  ZON          █████████░░░░░░░░░░░ 513 tokens 👑
               ├─ vs JSON formatted:  -45.4%
               ├─ vs JSON compact:    -12.9%
               ├─ vs TOON:            -16.4%
               ├─ vs CSV:             -3.9%
               ├─ vs YAML:            -29.5%
               └─ vs XML:             -53.1%

  TOON         ███████████░░░░░░░░░ 614 tokens 
               vs ZON: +19.7%

  CSV          ██████████░░░░░░░░░░ 534 tokens 
               vs ZON: +4.1%

  YAML         █████████████░░░░░░░ 728 tokens 
               vs ZON: +41.9%

  XML          ████████████████████ 1,093 tokens 
               vs ZON: +113.1%

  JSON (cmp)   ███████████░░░░░░░░░ 589 tokens 


🔹 Tokenizer: Claude 3.5 (Anthropic)
  ZON          ██████████░░░░░░░░░░ 548 tokens 
               ├─ vs JSON formatted:  -40.0%
               ├─ vs JSON compact:    -8.1%
               ├─ vs TOON:            -3.9%
               ├─ vs CSV:             +0.7%
               ├─ vs YAML:            -14.5%
               └─ vs XML:             -50.4%

  TOON         ██████████░░░░░░░░░░ 570 tokens 
               vs ZON: +4.0%

  CSV          ██████████░░░░░░░░░░ 544 tokens 👑
               vs ZON: -0.7%

  YAML         ████████████░░░░░░░░ 641 tokens 
               vs ZON: +17.0%

  XML          ████████████████████ 1,104 tokens 
               vs ZON: +101.5%

  JSON (cmp)   ███████████░░░░░░░░░ 596 tokens 


🔹 Tokenizer: Llama 3 (Meta)
  ZON          ██████████░░░░░░░░░░ 696 tokens 👑
               ├─ vs JSON formatted:  -43.1%
               ├─ vs JSON compact:    -8.4%
               ├─ vs TOON:            -11.2%
               ├─ vs CSV:             -4.4%
               ├─ vs YAML:            -22.1%
               └─ vs XML:             -50.0%

  TOON         ███████████░░░░░░░░░ 784 tokens 
               vs ZON: +12.6%

  CSV          ██████████░░░░░░░░░░ 728 tokens 
               vs ZON: +4.6%

  YAML         █████████████░░░░░░░ 894 tokens 
               vs ZON: +28.4%

  XML          ████████████████████ 1,392 tokens 
               vs ZON: +100.0%

  JSON (cmp)   ███████████░░░░░░░░░ 760 tokens 


════════════════════════════════════════════════════════════════════════════════
📊 Large Complex Nested Dataset
   Deeply nested, non-uniform structure with mixed types
────────────────────────────────────────────────────────────────────────────────
📦 BYTE SIZES:
  ZON:              335,611 bytes
  TOON:             607,194 bytes
  CSV:              369,682 bytes
  YAML:             607,189 bytes
  XML:              1,016,540 bytes
  JSON (formatted): 834,132 bytes
  JSON (compact):   551,854 bytes

🔹 Tokenizer: GPT-4o (o200k)
  ZON          █████████░░░░░░░░░░░ 143,661 tokens 👑
               ├─ vs JSON formatted:  -49.5%
               ├─ vs JSON compact:    -23.8%
               ├─ vs TOON:            -36.1%
               ├─ vs CSV:             -12.9%
               ├─ vs YAML:            -36.1%
               └─ vs XML:             -57.1%

  TOON         █████████████░░░░░░░ 224,940 tokens 
               vs ZON: +56.6%

  CSV          ██████████░░░░░░░░░░ 164,919 tokens 
               vs ZON: +14.8%

  YAML         █████████████░░░░░░░ 224,938 tokens 
               vs ZON: +56.6%

  XML          ████████████████████ 335,239 tokens 
               vs ZON: +133.4%

  JSON (cmp)   ███████████░░░░░░░░░ 188,604 tokens 


🔹 Tokenizer: Claude 3.5 (Anthropic)
  ZON          █████████░░░░░░░░░░░ 145,652 tokens 👑
               ├─ vs JSON formatted:  -46.8%
               ├─ vs JSON compact:    -21.3%
               ├─ vs TOON:            -26.0%
               ├─ vs CSV:             -9.9%
               ├─ vs YAML:            -26.0%
               └─ vs XML:             -55.5%

  TOON         ████████████░░░░░░░░ 196,893 tokens 
               vs ZON: +35.2%

  CSV          ██████████░░░░░░░░░░ 161,701 tokens 
               vs ZON: +11.0%

  YAML         ████████████░░░░░░░░ 196,892 tokens 
               vs ZON: +35.2%

  XML          ████████████████████ 327,274 tokens 
               vs ZON: +124.7%

  JSON (cmp)   ███████████░░░░░░░░░ 185,136 tokens 


🔹 Tokenizer: Llama 3 (Meta)
  ZON          ██████████░░░░░░░░░░ 230,838 tokens 👑
               ├─ vs JSON formatted:  -43.0%
               ├─ vs JSON compact:    -16.5%
               ├─ vs TOON:            -26.7%
               ├─ vs CSV:             -9.2%
               ├─ vs YAML:            -26.7%
               └─ vs XML:             -51.9%

  TOON         █████████████░░░░░░░ 314,824 tokens 
               vs ZON: +36.4%

  CSV          ███████████░░░░░░░░░ 254,181 tokens 
               vs ZON: +10.1%

  YAML         █████████████░░░░░░░ 314,820 tokens 
               vs ZON: +36.4%

  XML          ████████████████████ 480,125 tokens 
               vs ZON: +108.0%

  JSON (cmp)   ████████████░░░░░░░░ 276,405 tokens 


════════════════════════════════════════════════════════════════════════════════
📈 OVERALL SUMMARY
════════════════════════════════════════════════════════════════════════════════

🔹 GPT-4o (o200k) Summary:
  ZON Wins: 2/2 datasets
  Total Tokens:
  ZON: █████████████░░░░░░░░░░░░░░░░░ 144,174 tokens
       vs JSON (cmp): -23.8%
       vs TOON:       -36.1%
       vs CSV:        -12.9%
       vs YAML:       -36.1%
       vs XML:        -57.1%

🔹 Claude 3.5 (Anthropic) Summary:
  ZON Wins: 1/2 datasets
  Total Tokens:
  ZON: █████████████░░░░░░░░░░░░░░░░░ 146,200 tokens
       vs JSON (cmp): -21.3%
       vs TOON:       -26.0%
       vs CSV:        -9.9%
       vs YAML:       -26.0%
       vs XML:        -55.5%

🔹 Llama 3 (Meta) Summary:
  ZON Wins: 2/2 datasets
  Total Tokens:
  ZON: ██████████████░░░░░░░░░░░░░░░░ 231,534 tokens
       vs JSON (cmp): -16.5%
       vs TOON:       -26.6%
       vs CSV:        -9.2%
       vs YAML:       -26.7%
       vs XML:        -51.9%

════════════════════════════════════════════════════════════════════════════════
✨ Benchmark complete!
════════════════════════════════════════════════════════════════════════════════
