;; -*- mode: emacs-lisp; lexical-binding: t -*-

(defun dotspacemacs/layers ()
  (setq-default
   dotspacemacs-ask-for-lazy-installation t
   dotspacemacs-configuration-layers
   '(
     auto-completion
     emacs-lisp
     git
     (ivy
      :variables
      ivy-initial-inputs-alist nil)
     markdown
     multiple-cursors
     (python
      :variables
      python-formatter 'black
      python-format-on-save t
      python-sort-imports-on-save t)
     (shell
      :variables
      shell-default-height 30
      shell-default-position 'bottom)
     shell-scripts
     spell-checking
     syntax-checking
     treemacs
     version-control
     (xclipboard
      :variables
      xclipboard-enable-cliphist t))
   dotspacemacs-install-packages 'used-only
))

(defun dotspacemacs/init ()
  (setq-default
   dotspacemacs-default-font '("Fira Code")
   dotspacemacs-enable-server t
   dotspacemacs-startup-banner nil
   dotspacemacs-startup-buffer-show-version nil
   dotspacemacs-startup-lists '((recents . 16))
))

(defun dotspacemacs/user-config ()
  (add-hook 'tuareg-mode-hook #'eglot-ensure) ; for OCaml
  (add-hook 'python-mode-hook #'eglot-ensure) ; for Python

  (define-key evil-motion-state-map (kbd "C-z") 'suspend-frame)
  (define-key evil-emacs-state-map (kbd "C-z") 'suspend-frame)

  (load-file "~/.config/spacemacs/theme.el")
)
