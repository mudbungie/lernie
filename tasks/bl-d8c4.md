+++
title = "advisories.yml has failed daily at 'Mend the lockfile' since at least 2026-11-14 (yog too, since 2026-11-10; yog-android on 2026-11-17): the day's advisory is not being mended and nobody was told"
created = 1794984937
updated = 1794984937
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
The daily run goes red at the mend step in this repository and in yog; thrall's is green. Either an advisory has no fixed version (the designed hand's case) or the mend itself is broken. The run log could not be fetched with the gh client when this was filed, so the cause is unread: start by reading the failed step in the web view, or run `bl-remote-run advisories` and `make mend-advisories` on the builder.

Related, found in yog bl-37d8: the landing step's gh calls ran under the release token, which is refused a workflow dispatch (HTTP 403). This repository holds no such token so its file takes the job token already, but the line reads the same and should be checked against yog's fix.