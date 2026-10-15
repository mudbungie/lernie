+++
title = "a sent message is on the glass the frame it is sent: an operator row at the transcript's end in the pending ink until the engine acknowledges it, retired when the transcript carries it, back in the box when refused"
created = 1792034030
updated = 1792034030
parent = "bl-4792"
priority = 2
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"

[[blockers]]
id = "bl-b7cb"
on = "claim"
+++
DESIGN §4.45 item 4. composer::fire records what it composed (the conversation, the words, how many entries the transcript held) in the model beside the outbox; chat::rows appends each as an operator row after the committed rows and the live fold, its words in the pending ink while unacknowledged and in body ink once the act's receipt landed (Model::receipt on the message and interrupt ops); the row is dropped the frame the committed transcript carries an operator delivery with those words past the recorded length; a refusal drops it and puts the words back in the draft, as the start's refusal does; a lost reply leaves it standing in the pending ink beside the in-doubt notice. Keyed by conversation so a tab change keeps it with its own transcript. Tests: fire → row present in pending ink the same frame; receipt → body ink; transcript read carrying it → one row, not two; refusal → draft restored.