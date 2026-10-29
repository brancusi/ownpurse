(ns ownpurse.classify.outcome
  "How a coding fares at a confidence threshold. Shared by the eval on the JVM and the admin in the browser, so both
  score a line the same way.")

(def outcomes
  "How a coding fares, in the order the reports list them."
  [:right :wrong :unsafe :stopped :review])

(def unkeyed-outcomes
  "How a coding of a line with no answer key (a book's own) can fare: coded automatically, or sent to a person."
  [:coded :asks])

(defn outcome
  "How one coding fares at `threshold`. Coded automatically: :wrong when its bucket is not an acceptable one, else
  :unsafe when a person had to look, else :right. Sent to a person: :stopped when a person had to or may look,
  :review when it was needless. A line with no answer key is only :coded or :asks (sent to a person)."
  [{:keys [expect ok review]} {:keys [code confidence]} threshold]
  (let [acceptable (conj (set ok) expect)
        auto? (and (not= :needs-person code) (>= (or confidence 0) threshold))]
    (cond
      (nil? expect) (if auto? :coded :asks)
      (and auto? (not (acceptable code))) :wrong
      (and auto? (= :required review)) :unsafe
      auto? :right
      review :stopped
      :else :review)))
