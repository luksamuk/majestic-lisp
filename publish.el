(require 'ox-publish)
(require 'ox-gfm)
(require 'ox-html)

(setq *publishing-directory* (concat default-directory "doc/html/"))

(defun publish-subdir (dir)
  (concat *publishing-directory* dir))

(setq org-publish-project-alist
      `(
        ("static-images"
         :base-directory ,(concat default-directory "img/")
         :base-extension "png\\|jpg\\|jpeg"
         :publishing-directory ,(publish-subdir "img/")
         :recursive t
         :publishing-function org-publish-attachment)
        ("static-files"
         :base-directory ,(concat default-directory "static/")
         :base-extension "js\\|css"
         :publishing-directory ,*publishing-directory*
         :recursive t
         :publishing-function org-publish-attachment)        
        ("html"
         :base-directory ,default-directory
         :publishing-directory ,*publishing-directory*
         :base-extension "org"
         :exclude "setupfile.org"
         :publishing-function org-html-publish-to-html)
        ("tangle"
         :base-directory ,default-directory
         :publishing-directory ,default-directory
         :recursive t
         :exclude "static/\\|setupfile.org"
         :publishing-function org-babel-tangle-publish)
        ("majestic"
         :components ("static-images" "static-files" "html"))))
