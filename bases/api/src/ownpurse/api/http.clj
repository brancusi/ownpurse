(ns ownpurse.api.http
  "The HTTP server."
  (:require
   [ring.adapter.jetty9 :as jetty])
  (:import
   (org.eclipse.jetty.server NetworkConnector Server)))

(defn start
  "Starts Jetty with `handler` on `host`:`port` (0 picks a free port)."
  ^Server [{:keys [handler host port]}]
  (jetty/run-jetty handler {:host host :port port :join? false}))

(defn stop
  "Stops the server."
  [server]
  (jetty/stop-server server))

(defn port
  "The port the server is bound to."
  [^Server server]
  (.getLocalPort ^NetworkConnector (first (.getConnectors server))))
