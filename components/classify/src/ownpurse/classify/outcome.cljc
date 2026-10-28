(ns ownpurse.classify.outcome
  "How a coding fares at a confidence threshold. Shared by the eval on the JVM and the admin in the browser, so both
  score a line the same way.")

(def outcomes
  "How a coding fares, in the order the reports list them."
  [:right :wrong :unsafe :stopped :review])

(defn outcome
  "How one coding fares at `threshold`. Coded automatically: :wrong when its bucket is not an acceptable one, else
  :unsafe when a person had to look, else :right. Sent to a person: :stopped when a person had to or may look,
  :review when it was needless."
  [{:keys [expect ok review]} {:keys [code confidence]} threshold]
  (let [acceptable (conj (set ok) expect)
        auto? (and (not= :needs-person code) (>= confidence threshold))]
    (cond
      (and auto? (not (acceptable code))) :wrong
      (and auto? (= :required review)) :unsafe
      auto? :right
      review :stopped
      :else :review)))
