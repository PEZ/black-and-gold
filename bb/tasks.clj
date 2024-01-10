(ns tasks
  (:require
   [babashka.fs :as fs]
   [babashka.http-server :as server]
   [babashka.process :as p :refer [shell]]
   ;[cheshire.core :as json]
   ))

(def assets-dir "assets")

(def wasm-deploy-dir "deploy")

(def ios-app-name "Afuera")
(def ios-app-folder (fs/path "ios" (str ios-app-name ".app")))
(def ios-ipa (fs/path "ios" (str ios-app-name ".ipa")))

(def binary-name "afuera")

(defn wasm-build []
  (println "Building wasm target")
  (shell "cargo" "build" "--release" "--target" "wasm32-unknown-unknown"))

(defn wasm-deploy []
  (println "setting up deploy folder")
  (fs/delete-tree wasm-deploy-dir)
  (fs/create-dirs (fs/path "." wasm-deploy-dir assets-dir))
  (fs/copy-tree assets-dir (fs/path wasm-deploy-dir assets-dir))
  (fs/copy-tree (fs/path "wasm" "js") (fs/path wasm-deploy-dir "js"))
  (fs/copy (fs/path "target" "wasm32-unknown-unknown" "release" (str binary-name ".wasm"))
           wasm-deploy-dir)
  (fs/copy (fs/path "wasm" "index.html") wasm-deploy-dir))

(defn wasm-serve [opts]
  (server/exec (merge {:dir wasm-deploy-dir
                       :port 4000}
                      opts)))

(defn- ios-init-bundle [dev-or-dist]
  (fs/delete-tree ios-app-folder)
  (fs/create-dirs ios-app-folder)
  (fs/copy (fs/path "ios" "Info.plist") (fs/path ios-app-folder))
  (fs/copy (fs/path "ios" "entitlements.xml") (fs/path ios-app-folder))
  (doseq [png-file (fs/glob "." (fs/path "ios" "app-store-assets" "*.png"))]
    (fs/copy png-file (fs/path ios-app-folder)))
  (fs/copy (fs/path "ios" (str dev-or-dist ".mobileprovision"))
           (fs/path ios-app-folder "embedded.mobileprovision")))

(defn- ios-build-and-sign [signing-identity dev-or-dist]
  (ios-init-bundle dev-or-dist)
  (shell "cargo" "build" "--target" "aarch64-apple-ios" "--release")
  (fs/copy-tree assets-dir (fs/path ios-app-folder assets-dir))
  (fs/copy (fs/path "target" "aarch64-apple-ios" "release" binary-name) (fs/path ios-app-folder))
  (shell "codesign" "--force" "--timestamp=none" "--sign" signing-identity "--entitlements" "ios/entitlements.xml" ios-app-folder))

(defn ios-build-to-device [signing-identity device-id]
  (ios-build-and-sign signing-identity "dev")
  (shell "ios-deploy" #_"-d" "-i" device-id "-b" ios-app-folder))

(defn ios-build-ipa [signing-identity]
  (ios-build-and-sign signing-identity "dist")
  (fs/move ios-app-folder "Payload")
  (shell "zip" "-r" ios-ipa "Payload")
  (fs/move "Payload" ios-app-folder))

(defn ios-build-to-simulator []
  (ios-init-bundle "dev")
  (shell "cargo" "build" "--target" "x86_64-apple-ios" "--release")
  (fs/copy-tree assets-dir (fs/path ios-app-folder assets-dir))
  (fs/copy (fs/path "target" "x86_64-apple-ios" "release" binary-name) (fs/path ios-app-folder))
  (shell "xcrun" "simctl" "install" "booted" ios-app-folder)
  (shell "xcrun" "simctl" "launch" "booted" (str "news.afuera." ios-app-name)))

(defn- ios-validate-or-upload-ipa [validate-or-upload apple-id password]
  (shell "xcrun" "altool"
         (validate-or-upload {:validate "--validate-app" :upload "--upload-app"})
         "-f" ios-ipa "-t" "ios" "-u" apple-id "-p" password))

(defn ios-validate-ipa [apple-id password]
  (ios-validate-or-upload-ipa :validate apple-id password))

(defn ios-upload-ipa [apple-id password]
  (ios-validate-or-upload-ipa :upload apple-id password))

(comment
  (ios-build-to-device "3E2ADFBFE34A2CBA6D6423DD5CD0E9BD944C7897" "00008110-0008152C2E92801E")
  (ios-init-bundle "dev")
  (wasm-deploy)
  (wasm-serve {})
  :rcf)
