# VC Frame peers

In vc-frame 4.5.1 every workspace is its own session. A host and a project each
own a compact bar, a session rail, and a status bar. Choosing another workspace
switches the frontend to that session. It does not replace a center pane inside
the host, and this release has no `VC Guest` pane and no `vibecrafted-guest`
layout.

`visit` is a normal read/write client, unlike `watch`. From inside Frame it
switches the current frontend. From an external terminal it attaches the named
session. Closing that client disconnects only the client; the session server
and its PTYs keep running.

## Switch to an existing session

From inside Frame:

```sh
vc-frame visit OLD_SESSION --tab 1
```

The legacy spelling does the same peer switch. `--session` names the source
frontend, not the owner of an embedded guest:

```sh
vc-frame --session SOURCE project-workspace OLD_SESSION --tab 1
```

Tab numbers are one-based. Both commands send `SwitchSession` when the source
and target differ. When they are the same session and a tab was requested, the
command sends `GoToTab` and does not reconnect. Pass `--client-id` when more
than one frontend is attached and the request must hit a specific client.

This keeps the target server, its terminal PIDs, PTYs, provider processes, and
conversations. It changes which session the frontend is attached to. It is not
a server upgrade. Do not kill, resurrect, re-exec, or recreate a live session
to put a different one on screen.

Detach the previous visual client through its own UI, or close its terminal
window, after the new frontend is up. To return, attach the previous session
with its own client and select the tab there. A transient
`vc-frame action go-to-tab` targets the last active client and is the wrong
tool when another client is still connected.

## What this release does not do

Embedded guest projection is not available in 4.5.1. The older procedure below
is historical. Following it does not keep host chrome wrapped around another
session:

- `vc-frame --layout vibecrafted-guest attach --create-background my-work`
- opening `vibecrafted-host` and expecting a rail selection to replace only a
  center `VC Guest` pane
- reading `project-workspace` as an embedded visitor that publishes host chrome
  over a guest pane

`--guest-workspace` still selects a layout, and that layout stays a complete
peer with its own session chrome. It is not a content-only pane mounted inside
another session.

## Private regression against an older binary

`zellij-server/tests/workspace_chrome_pty.py` can still exercise an older
binary's ten-tab chrome through a current binary. That harness is a private
regression, not the operator entry above. It needs the old binary, the old
layout, and an explicit rail config:

```sh
uv run --with pyte python zellij-server/tests/workspace_chrome_pty.py \
  --binary /absolute/new/vc-frame --legacy-binary /absolute/old/vc-frame \
  --rail-config /absolute/config.kdl --bridge-rounds 2 \
  --output /absolute/new-evidence-directory
```

`--legacy-product-layout` and `--legacy-config` keep an older product host's
own bars and work. They do not turn `project-workspace` in this release into
an embedded projection. Do not kill live work to satisfy the harness.
