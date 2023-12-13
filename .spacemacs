;; -*- mode: emacs-lisp; lexical-binding: t -*-

(defun dotspacemacs/layers ()
  (setq-default
   dotspacemacs-ask-for-lazy-installation t

   dotspacemacs-configuration-layers
   '(
     (auto-completion
      :variables
      auto-completion-idle-delay 0)
     better-defaults
     emacs-lisp
     git
     (ivy
      :variables
      ivy-initial-inputs-alist nil)
     markdown
     (python
      :variables
      python-formatter 'black
      python-format-on-save t
      python-sort-imports-on-save t)
     (shell
      :variables
      shell-default-height 30
      shell-default-position 'bottom)
     spell-checking
     syntax-checking
     shell-scripts
     version-control
     treemacs)
   dotspacemacs-install-packages 'used-only
))

(defun dotspacemacs/init ()
  (setq-default
   dotspacemacs-enable-server t
   dotspacemacs-line-numbers t
   dotspacemacs-startup-banner nil
   dotspacemacs-startup-buffer-multi-digit-delay 0.0
   dotspacemacs-startup-buffer-show-version nil
   dotspacemacs-startup-lists '((recents . 16))
   dotspacemacs-which-key-delay 0.0

   dotspacemacs-default-font '("Fira Code"
                               :size 10.0
                               :weight normal
                               :width normal)
))

(defun dotspacemacs/user-config ()
  (add-hook 'tuareg-mode-hook #'eglot-ensure) ; for OCaml
  (add-hook 'python-mode-hook #'eglot-ensure) ; for Python

  (define-key evil-motion-state-map (kbd "C-z") 'suspend-frame)
  (define-key evil-emacs-state-map (kbd "C-z") 'suspend-frame)
  (define-key evil-motion-state-map (kbd "s") 'evil-avy-goto-word-0)

  (load-file "~/.config/spacemacs/theme.el")
)
