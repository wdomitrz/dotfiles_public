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
))

(defun dotspacemacs/user-config ()
  (setq initial-buffer-choice t)

  (define-key evil-motion-state-map (kbd "C-z") 'suspend-frame)
  (define-key evil-emacs-state-map (kbd "C-z") 'suspend-frame)

  (load-file "~/.config/spacemacs/theme.el")
)
