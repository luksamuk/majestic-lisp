(require 'ox-publish)
(require 'ox-gfm)

(setq org-publish-project-alist
      `(("majestic-book"
         :base-directory ,default-directory
         :publishing-directory ,(concat default-directory "doc/src/")
         :base-extension "org"
         :publishing-function org-gfm-publish-to-gfm)))
