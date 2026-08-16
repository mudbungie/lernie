+++
title = "0.0.10 release prep: promote the [Unreleased] changelog section and merge the release PR"
created = 1786844602
updated = 1786844602
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
The release ships the two chat-wedge fixes (bl-15f0 alternation-wide warrant, bl-4187 crash settlement at the drive boundary) plus bl-9001's tool-injection seam and bl-8f7c's restored bullet. Promote [Unreleased] to [0.0.10] via make promote-changelog, land it, approve the release PR's CI run, merge, and verify the release job tags v0.0.10 and publishes.