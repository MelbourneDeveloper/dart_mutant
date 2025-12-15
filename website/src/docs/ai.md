---
layout: docs.njk
title: AI-Powered Mutations
---

# AI-Powered Mutations

dart_mutant can optionally use AI to identify high-value mutation locations.

## Overview

Traditional mutation testing generates mutations at every possible location. AI-powered mode uses language models to:

- Identify code that's most likely to contain bugs
- Focus on complex business logic
- Skip trivial mutations
- Suggest custom mutations beyond standard operators

## Supported Providers

### Anthropic (Claude)

```bash
export ANTHROPIC_API_KEY=your_api_key
dart_mutant --ai anthropic
```

### OpenAI (GPT)

```bash
export OPENAI_API_KEY=your_api_key
dart_mutant --ai openai
```

### Ollama (Local)

Run AI locally without API keys:

```bash
# Start Ollama with a code model
ollama run codellama

# Use with dart_mutant
dart_mutant --ai ollama --ollama-model codellama
```

#### Custom Ollama URL

```bash
dart_mutant --ai ollama --ollama-url http://localhost:11434 --ollama-model codellama
```

## How It Works

1. **Code Analysis**: AI analyzes each file to identify:
   - Complex conditional logic
   - Edge case handling
   - Error-prone patterns
   - Business-critical calculations

2. **Mutation Prioritization**: High-value locations are tested first

3. **Custom Mutations**: AI can suggest mutations beyond standard operators:
   - Off-by-one errors
   - Boundary condition bugs
   - Type confusion
   - Race conditions

## Example Output

```
  AI Analysis: lib/src/payment_processor.dart
  ├─ High priority: validateCard() - complex validation logic
  ├─ High priority: calculateTotal() - financial calculation
  ├─ Medium priority: formatReceipt() - string formatting
  └─ Low priority: logTransaction() - logging only

  Generating mutations for high-priority locations first...
```

## When to Use AI Mode

### Good Use Cases

- **Large codebases**: Focus testing time on important code
- **Time-constrained CI**: Get valuable results in less time
- **Complex domains**: Finance, healthcare, security-critical code

### Not Recommended

- **Small projects**: Standard mutation is fast enough
- **Air-gapped environments**: Requires API access (except Ollama)
- **Cost-sensitive**: API calls have costs

## Cost Considerations

### Anthropic/OpenAI

- Charged per token analyzed
- Typical project (10k lines): ~$0.10-0.50
- Cache results to avoid repeated analysis

### Ollama (Free)

- Runs locally, no API costs
- Requires GPU for good performance
- Slower than cloud APIs

## Configuration

### Combining with Standard Options

```bash
# AI-prioritized mutations with threshold
dart_mutant --ai anthropic --threshold 80

# AI analysis + sampling
dart_mutant --ai anthropic --sample 100

# Local AI + verbose output
dart_mutant --ai ollama --verbose
```

### Caching AI Analysis

AI analysis results are cached by default:

```
.dart_mutant_cache/
└── ai_analysis/
    ├── payment_processor.dart.json
    └── user_service.dart.json
```

Clear cache to re-analyze:

```bash
rm -rf .dart_mutant_cache/ai_analysis
```

## Best Practices

1. **Start without AI**: Understand your baseline mutation score first
2. **Use for prioritization**: Let AI identify where to focus improvement
3. **Review suggestions**: AI isn't perfect - validate recommendations
4. **Cache results**: Avoid repeated API calls for unchanged files

## Privacy Note

When using cloud AI providers:

- Your source code is sent to the AI service
- Consider using Ollama for sensitive codebases
- Review provider privacy policies

## Next Steps

- [CLI Options](/docs/cli/) - All command-line options
- [Interpreting Results](/docs/interpreting/) - Understanding AI-suggested mutations
