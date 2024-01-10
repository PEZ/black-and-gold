(ns tasks
  (:require
   [babashka.fs :as fs]
   [babashka.http-server :as server]
   [babashka.process :as p :refer [shell]]
   ;[cheshire.core :as json]
   ))

(def wasm-deploy-dir "deploy")

(defn wasm-build []
  (println "Building wasm target")
  (shell "cargo" "build" "--release" "--target" "wasm32-unknown-unknown"))

(defn wasm-deploy []
  (println "setting up deploy folder")
  (fs/create-dirs (fs/path "." wasm-deploy-dir "assets"))
  (fs/copy-tree "assets"
                (fs/path wasm-deploy-dir "assets") {:replace-existing true})
  (fs/copy-tree (fs/path "wasm" "js")
                (fs/path wasm-deploy-dir "js") {:replace-existing true})
  (fs/copy (fs/path "target" "wasm32-unknown-unknown" "release" "afuera.wasm")
           wasm-deploy-dir {:replace-existing true})
  (fs/copy (fs/path "wasm" "index.html")
           wasm-deploy-dir {:replace-existing true}))

(defn wasm-serve [opts]
  (server/exec (merge {:dir wasm-deploy-dir
                       :port 4000}
                      opts)))

(comment
  (wasm-deploy)
  (wasm-serve {})
  :rcf)
