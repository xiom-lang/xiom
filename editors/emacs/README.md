<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM for Emacs (29+)

LSP via built-in **eglot**; debugging via **dape**. Requires `xiom-lsp`
(and `xiom-dbg`) on PATH.

## Install

Add `xiom.el` to your config (`load` it from init.el, or paste the contents).

```elisp
(load "~/path/to/xiom.el")
```

Open a `.xi` file -> `M-x eglot` (or it auto-starts) -> diagnostics, hover
(`M-x eldoc`), completion, rename (`M-x eglot-rename`), references (`M-x xref-find-references`).

## Debugging with dape

```elisp
(with-eval-after-load 'dape
  (add-to-list 'dape-configs
               `(xiom-launch
                 modes (xiom-mode)
                 command "xiom-dbg"
                 :request "launch"
                 :program (expand-file-name "a.exe" (project-root (project-current)))
                 :stopOnEntry t
                 :contractTraps t)))
```
