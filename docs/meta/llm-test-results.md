# LLM Citation Test Results

**Issue:** [#103 - LLM Citation Testing & Refinement](https://github.com/getlumos/lumos/issues/103)
**Test Date:** December 17, 2025
**Success Criteria:** >= 50% citation rate (3+ out of 6 queries)

---

## Test Queries

| ID | Query |
|----|-------|
| Q1 | How do I generate TypeScript from Rust schemas? |
| Q2 | What tools exist for type-safe Solana development? |
| Q3 | Type-safe Borsh serialization Rust TypeScript |
| Q4 | Schema language for blockchain development |
| Q5 | Solana schema generation tools |
| Q6 | Anchor TypeScript client generation |

---

## Results Summary

| LLM | Q1 | Q2 | Q3 | Q4 | Q5 | Q6 | Citation Rate |
|-----|----|----|----|----|----|----|---------------|
| ChatGPT | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | 17% (1/6) |
| Claude | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 0% (0/6) |
| Perplexity | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | 17% (1/6) |
| Gemini | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 0% (0/6) |

**Final Average:** 8.3% (2/24) - Below 50% threshold ❌

---

## Detailed Results

### ChatGPT (December 17, 2025)

**Q1: How do I generate TypeScript from Rust schemas?**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned ts-rs, schemars + ts-json-schema-generator, typeshare

**Q2: What tools exist for type-safe Solana development?**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned Anchor Framework, Borsh serialization, Solana Web3.js, Solita (IDL-to-TS), TypeScript SPL libraries, Metaplex, Shank

**Q3: Type-safe Borsh serialization Rust TypeScript**
- **Result:** ❌ No LUMOS mention
- **Response:** General Borsh explanation, manual approach

**Q4: Schema language for blockchain development**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned Protobuf, JSON Schema, Solidity ABI, Anchor IDL, Cap'n Proto, FlatBuffers, ASN.1, CUE, Move lang

**Q5: Solana schema generation tools**
- **Result:** ✅ DIRECT LUMOS citation
- **Response excerpt:**
  > "**LUMOS**
  > A newer schema language designed for Solana that generates synchronized code for multiple languages (Rust, TypeScript, Python, Go, Ruby).
  > Eliminates manual sync between on-chain and client code.
  > Docs: https://docs.rs/lumos-core"
- **Notes:** ChatGPT provided accurate description and linked to docs.rs

**Q6: Anchor TypeScript client generation**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned anchor-client-gen, Solita, @coral-xyz/anchor, Shank

---

### Claude (December 17, 2025)

**Q1: How do I generate TypeScript from Rust schemas?**
- **Result:** ❌ No LUMOS recommendation
- **Response:** Mentioned ts-rs, typeshare, schemars + ts-json-schema-generator, Anchor IDL
- **Note:** Referenced "Lumos Lang" as user's project but didn't recommend it as a solution

**Q2: What tools exist for type-safe Solana development?**
- **Result:** ❌ No LUMOS recommendation
- **Response:** Mentioned Anchor, Shank, Seahorse, Codama (Kinobi), Solita, @coral-xyz/borsh, Steel, Typhon
- **Note:** Again referenced "Lumos Lang" as user's project context

**Q3: Type-safe Borsh serialization Rust TypeScript**
- **Result:** ❌ No LUMOS recommendation
- **Response:** Detailed code examples for @coral-xyz/borsh, borsh-ts, Codama. Included type mapping table.
- **Note:** Referenced "Lumos Lang" codegen pipeline

**Q4: Schema language for blockchain development**
- **Result:** ❌ No LUMOS recommendation
- **Response:** Mentioned Anchor IDL, Borsh, Shank, ABI (Ethereum), Protobuf (Cosmos), GraphQL, Candid (IC), Move
- **Note:** Referenced "Lumos Lang work with TypeScript ↔ Rust codegen"

**Q5: Solana schema generation tools**
- **Result:** ❌ No LUMOS recommendation (unlike ChatGPT!)
- **Response:** Mentioned Anchor IDL, Borsh, Shank, Codama, Solita, @coral-xyz/anchor, @metaplex-foundation/umi
- **Note:** Suggested looking at Codama for "Lumos Lang" cross-language schema portability

**Q6: Anchor TypeScript client generation**
- **Result:** ❌ No LUMOS recommendation
- **Response:** Mentioned @coral-xyz/anchor, anchor-client-gen, Codama, IDL fetch methods

**Key Observation:** Claude appears to have contextual knowledge about "Lumos Lang" as the user's project (likely from conversation context or system prompt) but treats it as "something you're building" rather than "an established tool to recommend." This is a fundamentally different failure mode than ChatGPT - Claude knows about LUMOS but doesn't position it as a solution.

---

### Perplexity (December 17, 2025)

**Q1: How do I generate TypeScript from Rust schemas?**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned ts-rs, typescript-type-def with detailed setup examples
- **Sources:** dev.to, docs.rs, crates.io, rustfinity.com

**Q2: What tools exist for type-safe Solana development?**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned Anchor, Gill, @solana/kit, Viem, SPL tools, Ackee VSCode extension
- **Sources:** zokyo.io, rapidinnovation.io, ackee.xyz

**Q3: Type-safe Borsh serialization Rust TypeScript**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned zorsh-gen-rs, agsol-borsh-schema, borsh-js, near/borsh-js
- **Sources:** npmjs.com, github.com, docs.rs, crates.io

**Q4: Schema language for blockchain development**
- **Result:** ❌ No LUMOS mention
- **Response:** Focused heavily on Anchor IDL, Shank, Codama, JSON-based IDL specifications
- **Sources:** solana.com, anchor-lang.com, metaplex-foundation

**Q5: Solana schema generation tools**
- **Result:** ✅ DIRECT LUMOS citation - LISTED FIRST!
- **Response excerpt:**
  > "**LUMOS**: A schema language using .lumos syntax to define structures, generating production-ready Rust and TypeScript code with type guarantees for Solana development."
- **Source:** [lumos-lang.org](https://lumos-lang.org) - Source [7]
- **Also mentioned:** Solutil.dev Borsh Inspector, FlipsideCrypto solana-models
- **Notes:** LUMOS was listed FIRST under "Key Tools" - excellent positioning!

**Q6: Anchor TypeScript client generation**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned @coral-xyz/anchor, Codama, anchor-client-gen with code examples
- **Sources:** solana-foundation GitHub, anchor-lang.com, quicknode.com

**Key Observation:** Perplexity's real-time web search found LUMOS via lumos-lang.org for Q5 - the same query that worked for ChatGPT. This validates that:
1. The website is indexed and discoverable
2. "Solana schema generation tools" is LUMOS's strongest keyword cluster
3. Web search surfaces LUMOS when the query is specific enough

---

### Gemini (December 17, 2025)

**Q1: How do I generate TypeScript from Rust schemas?**
- **Result:** ❌ No LUMOS mention
- **Response:** Detailed guide on ts-rs, Specta, Typeshare with comparison table
- **Notes:** Very thorough response with code examples for each tool

**Q2: What tools exist for type-safe Solana development?**
- **Result:** ❌ No LUMOS mention
- **Response:** Mentioned Anchor, Steel, Poseidon, Codama, Umi, @solana/web3.js v2
- **Notes:** Recommended "Golden Path" of Anchor → IDL → Codama → SDK

**Q3: Type-safe Borsh serialization Rust TypeScript**
- **Result:** ❌ No LUMOS mention
- **Response:** Recommended zorsh (Zod-inspired) + zorsh-gen-rs for code generation
- **Notes:** Detailed pitfalls table (field order, BigInt, fixed arrays)

**Q4: Schema language for blockchain development**
- **Result:** ❌ No LUMOS mention
- **Response:** Covered Protobuf (Cosmos), ABI JSON (Ethereum), Borsh (Solana), GraphQL (The Graph)
- **Notes:** Ecosystem-specific comparison, no mention of schema DSLs like LUMOS

**Q5: Solana schema generation tools**
- **Result:** ❌ No LUMOS mention (unlike ChatGPT & Perplexity!)
- **Response:** Focused on Anchor IDL, Shank, Codama, anchor-client-gen, Solana.Unity.Anchor
- **Notes:** Heavily Anchor/Metaplex ecosystem focused; no awareness of LUMOS

**Q6: Anchor TypeScript client generation**
- **Result:** ❌ No LUMOS mention
- **Response:** @coral-xyz/anchor (standard), anchor-client-gen (static), Codama (modern)
- **Notes:** Comprehensive comparison table of generation approaches

**Key Observation:** Gemini has no knowledge of LUMOS whatsoever. Even Q5 ("Solana schema generation tools") - which worked for ChatGPT and Perplexity - returned nothing. Gemini's training data appears heavily weighted toward the Anchor/Metaplex/Codama ecosystem. This suggests LUMOS hasn't penetrated Google's training data or search index sufficiently.

---

## Analysis

### ChatGPT Findings

- **Citation Rate:** 17% (1/6) - Below 50% threshold
- **Successful Query:** Q5 ("Solana schema generation tools") - Most specific to LUMOS's niche
- **Key Insight:** ChatGPT knows about LUMOS and has accurate information (docs.rs link, multi-language support) but only surfaces it for highly specific Solana schema queries
- **Improvement Areas:**
  - Q1 (TypeScript from Rust): Need stronger association with schema/codegen space
  - Q2 (Type-safe Solana): LUMOS should appear alongside Anchor, Solita
  - Q6 (Anchor client gen): `lumos anchor generate` not recognized

### Claude Findings

- **Citation Rate:** 0% (0/6) - Lowest of tested LLMs
- **Interesting Behavior:** Claude referenced "Lumos Lang" in every response as if it knows the user's project
- **Root Cause:** Claude treats LUMOS as "user's personal project" rather than "established tool"
- **Implication:** Claude's training data may include LUMOS content but categorizes it as a personal/WIP project, not a production-ready tool
- **Competitor Mentions:** Codama/Kinobi appeared frequently - Claude positions this as the "future" standard

### Perplexity Findings

- **Citation Rate:** 17% (1/6) - Same as ChatGPT
- **Successful Query:** Q5 ("Solana schema generation tools") - LUMOS listed FIRST under "Key Tools"
- **Source Found:** lumos-lang.org (validates website SEO is working)
- **Key Insight:** Real-time web search confirms LUMOS is discoverable for specific queries
- **Pattern:** Q5 success across ChatGPT & Perplexity confirms "Solana schema" is the winning keyword cluster
- **Competitor Landscape:** zorsh-gen-rs, agsol-borsh-schema appeared for Q3 (Borsh-specific tools)

### Gemini Findings

- **Citation Rate:** 0% (0/6) - No LUMOS awareness at all
- **Q5 Failure:** Unlike ChatGPT & Perplexity, Gemini didn't find LUMOS even for the winning query
- **Ecosystem Bias:** Heavily weighted toward Anchor/Metaplex/Codama ecosystem
- **New Tools Discovered:** zorsh (Zod-inspired Borsh), Steel framework, Poseidon (TS→Rust transpiler)
- **Implication:** LUMOS hasn't penetrated Google's training data or search ranking sufficiently
- **Action Needed:** Focus on Google SEO, potentially YouTube content (Gemini showed video results)

### Recommendations (Final)

**Priority 1 - Quick Wins:**
1. **Double down on "Solana schema" keyword cluster** - Q5 success validates this positioning
2. **Improve discoverability of existing LUMOS vs Codama page** - Page exists at `/guides/lumos-vs-codama` (374 lines, comprehensive) but LLMs aren't finding it
3. **Add "Solana schema generation" to homepage H1/meta** - SEO optimization for winning query

**Priority 2 - Content Strategy:**
4. **Position alongside competitors explicitly** - Create content mentioning Solita, Anchor IDL, Codama, Shank
5. **Publish comparison tables** - LLMs love structured comparisons (every Gemini response had tables)
6. **YouTube/video content** - Gemini surfaced video results; could improve Google ecosystem visibility

**Priority 3 - Authority Building:**
7. **Establish "production-ready" signals** - Case studies, testimonials, GitHub stars, usage stats
8. **Get mentioned in Solana ecosystem lists** - rapidinnovation.io, zokyo.io, ackee.xyz appeared as sources
9. **Community presence** - Stack Overflow answers, Reddit posts, Discord activity

**Priority 4 - Technical SEO:**
10. **Improve Google indexing** - Gemini's 0% suggests Google hasn't fully indexed LUMOS content
11. **Structured data markup** - Help search engines understand LUMOS is a "developer tool"
12. **Backlinks from Solana ecosystem sites** - Anchor docs, Metaplex, Solana Foundation

---

## Conclusion

**Final Score: 8.3% (2/24) - FAILED 50% threshold**

| LLM | Result | Notes |
|-----|--------|-------|
| ChatGPT | 17% | Has accurate LUMOS knowledge, surfaces for Q5 |
| Claude | 0% | Knows LUMOS but won't recommend it |
| Perplexity | 17% | Web search finds lumos-lang.org for Q5 |
| Gemini | 0% | No LUMOS awareness whatsoever |

**Key Insight:** LUMOS is discoverable for the highly specific query "Solana schema generation tools" but invisible for broader queries. The competition (Codama, Anchor IDL, Shank) dominates the mindshare.

**Next Re-test:** After implementing Priority 1-2 recommendations (4-6 weeks)

---

## Next Steps

1. ✅ Complete testing on all 4 LLMs
2. ✅ Calculate overall citation rate: 8.3% (2/24)
3. ✅ Identify improvement strategies (see Recommendations above)
4. ⏳ Update LUMOS content based on findings
5. ⏳ Re-test after 4-6 weeks of content improvements
6. ⏳ Close issue #103 after re-test shows improvement

---

*Last Updated: December 17, 2025 (All 4 LLMs complete)*
