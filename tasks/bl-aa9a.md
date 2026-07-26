+++
title = "experiments/baseline/workflow.yaml duplicates template/workflow.yaml — derive the baseline, don't copy it"
created = 1785028722
updated = 1785028765
claimant = "Fennel"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Bracken finding from bl-0e79: experiments/baseline/workflow.yaml is a byte-identical hand-copy of template/workflow.yaml — one fact in two homes, and it had already drifted-in-waiting (both copies carried the same dead verb). The bl-0e79 vocabulary sweep now covers experiments/ so drift FAILS tests, but the duplication itself remains. Per experiments/README.md an experiment is 'a config diff, no code changes' — the baseline experiment IS the template, so it should derive, not duplicate: e.g. baseline/ holds no workflow.yaml and agent-eval falls back to the template bytes (the embedded include_dir! copy), or baseline/workflow.yaml becomes a generated artifact with a golden test like schemas/. Pick the arm that keeps 'experiment = diff against the template' literally true. Update experiments/README.md.