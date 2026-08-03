+++
title = "0.0.6 release prep: promote the changelog [Unreleased] section to [0.0.6]"
created = 1785737257
updated = 1785737257
claimant = "release-006"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Run 'make promote-changelog VERSION=0.0.6' in the claimed worktree and deliver it to main, mirroring bl-7a63's 0.0.5 prep. The [Unreleased] section already carries the release's two bullets (bl-93e6, the wait/do-it-yourself prompt surfaces; bl-a96a, reply-vs-obituary deposit addressing); this ball only stamps the section as [0.0.6] before release PR #6 merges, so the tagged tree's CHANGELOG.md is the record (release-plz.yml header ordering; changelog is hand-maintained, changelog_update=false, bl-7558).