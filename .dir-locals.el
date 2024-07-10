;;; Directory Local Variables            -*- no-byte-compile: t -*-
;;; For more information see (info "(emacs) Directory Variables")

((rust-ts-mode . ((compile-command . "cargo build")
                  (eglot-server-programs
                   . (((rust-ts-mode)
                       . ("/gnu/store/2id0kavrd79dkrx87n16mmbikswafc4v-profile/bin/rust-analyzer")))))))
