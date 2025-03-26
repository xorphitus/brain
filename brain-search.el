;;; brain-search.el --- Search brain knowledge base with Consult -*- lexical-binding: t -*-

;; Author: Cline
;; Version: 0.1.0
;; Package-Requires: ((emacs "27.1") (consult "0.16"))
;; Keywords: convenience
;; URL: https://github.com/xorphitus/brain

;;; Commentary:

;; This package provides integration between the brain CLI tool and Emacs,
;; allowing users to search their knowledge base and select files using Consult.
;;
;; Usage:
;;   M-x brain-search
;;   Enter a query when prompted
;;   Select a file from the results using Consult

;;; Code:

(require 'consult)

(defgroup brain-search nil
  "Search brain knowledge base with Consult."
  :group 'convenience
  :prefix "brain-search-")

(defcustom brain-search-command "brain"
  "Path to the brain command."
  :type 'string
  :group 'brain-search)

(defcustom brain-search-jq-command "jq"
  "Path to the jq command."
  :type 'string
  :group 'brain-search)

;;;###autoload
(defun brain-search ()
  "Search brain knowledge base and select a file with Consult.
Interactively gets a query, runs brain search, and presents results with Consult."
  (interactive)
  (let* ((query (read-string "Brain search query: "))
         (shell-command (format "%s --mode search-only --format json %s | %s -r '.matched_files.[].path'"
                               brain-search-command
                               (shell-quote-argument query)
                               brain-search-jq-command))
         (output (shell-command-to-string shell-command))
         (paths (split-string output "\n" t)))
    (if (null paths)
        (message "No matching files found for query: %s" query)
      (let ((selected (consult--read paths
                                    :prompt "Select file: "
                                    :category 'file
                                    :sort nil
                                    :require-match t
                                    :preview-key consult-preview-key
                                    :state (consult--file-preview)
                                    :history 'brain-search-history)))
        (when selected
          (find-file selected))))))

(provide 'brain-search)
;;; brain-search.el ends here
