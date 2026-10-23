(ns ownpurse.money.core)

(def ^:private decimal-string #"-?\d+(\.\d+)?")

(defn amount? [x]
  (decimal? x))

(defn- float-money? [x]
  (or (instance? Double x) (instance? Float x)))

(defn amount
  [x]
  (cond
    (decimal? x) x
    (int? x) (bigdec x)
    (instance? clojure.lang.BigInt x) (bigdec x)
    (instance? java.math.BigInteger x) (bigdec x)
    (and (string? x) (re-matches decimal-string x)) (BigDecimal. ^String x)

    (float-money? x)
    (throw (ex-info (str x " is a float. Amounts are exact decimals: send the digits as a string (\"12.50\") or "
                         "a BigDecimal (12.50M).")
                    {:type :ownpurse/float-money :value x}))

    :else
    (throw (ex-info (str (pr-str x) " is not an amount. Send a decimal string such as \"-12.50\" or a BigDecimal.")
                    {:type :ownpurse/not-an-amount :value x}))))

(defn totals [amounts]
  (reduce (fn [acc {:keys [amount commodity]}]
            (update acc commodity (fnil + 0M) amount))
          {}
          amounts))

(defn imbalance [amounts]
  (into {} (remove (fn [[_ total]] (zero? total))) (totals amounts)))

(defn balanced? [amounts]
  (empty? (imbalance amounts)))

(defn float-paths [x]
  (letfn [(walk [path x]
            (cond
              (float-money? x) [path]
              (map? x) (mapcat (fn [[k v]] (walk (conj path k) v)) x)
              (set? x) (mapcat #(walk (conj path %) %) x)
              (sequential? x) (mapcat (fn [i v] (walk (conj path i) v)) (range) x)
              :else []))]
    (vec (walk [] x))))
