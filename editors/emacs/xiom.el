;;; xiom.el --- XIOM language support via eglot -*- lexical-binding: t; -*-
;; Requires Emacs 29+ (eglot built in) and xiom-lsp on PATH.

;; Basic major mode for .xi files (prog-mode derivative with // comments)
(define-derived-mode xiom-mode prog-mode "XIOM"
  "Major mode for the XIOM programming language."
  (setq-local comment-start "// ")
  (setq-local comment-end "")
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width 2))

(add-to-list 'auto-mode-alist '("\\.xi\\'" . xiom-mode))

;; Wire eglot to xiom-lsp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(xiom-mode . ("xiom-lsp"))))

;; Auto-start eglot in xiom buffers
(add-hook 'xiom-mode-hook #'eglot-ensure)

(provide 'xiom)
;;; xiom.el ends here
