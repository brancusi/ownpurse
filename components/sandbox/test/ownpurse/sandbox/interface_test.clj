(ns ownpurse.sandbox.interface-test
  (:require
   [clojure.java.io :as io]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.sandbox.interface :as sandbox])
  (:import
   (java.nio.file Files)
   (java.nio.file.attribute FileAttribute)))

(def runs "components/classify/runs")

(defn- newest [sb scenario]
  (:id (first (:runs (some #(when (= scenario (:name %)) %) (sandbox/scenarios sb))))))

(deftest scenarios-test
  (let [sb (sandbox/open {:dir runs})
        harbour (some #(when (= "harbour" (:name %)) %) (sandbox/scenarios sb))]
    (testing "ownpurse's scenarios, each with its size and its runs, newest first"
      (is (= {:lines 51 :units 65} (select-keys harbour [:lines :units])))
      (is (seq (:runs harbour)))
      (is (= (:runs harbour) (sort-by :id #(compare %2 %1) (:runs harbour)))))
    (testing "each scenario's made-up lines are a set of lines to pick, read by its own rules"
      (is (= {:id "sample/harbour" :kind "sample" :lines 51 :units 65 :scenarios ["harbour"]}
             (select-keys (some #(when (= "sample/harbour" (:id %)) %) (sandbox/line-sets sb))
                          [:id :kind :lines :units :scenarios])))
      (let [{:keys [peek first from to currencies]} (sandbox/line-set sb "sample/harbour")]
        (is (= 3 (count peek)))
        (is (= 51 (count first)) "every line, up to two hundred")
        (is (= (sort (map :date first)) (map :date first)) "oldest first")
        (is (= from (:date (clojure.core/first first))))
        (is (string? to))
        (is (= 51 (reduce + (vals currencies))))))
    (testing "a scenario's world is everything but its lines"
      (let [w (sandbox/world sb "harbour")]
        (is (nil? (:cases w)))
        (is (seq (:rules w)))
        (is (seq (:buckets w)))))))

(deftest run-and-line-test
  (let [sb (sandbox/open {:dir runs})
        id (newest sb "harbour")
        {:keys [run lines]} (sandbox/run-lines sb "harbour" id)
        first-id (:id (first lines))]
    (testing "a run is one light row a unit, with each strategy's coding"
      (is (= 65 (count lines) (:results run)))
      (is (every? #(get-in % [:decisions :choice :code]) lines))
      (is (every? #(contains? (:unit %) :expect) lines)))
    (testing "a line comes with its facts, the requests it made, the answers, and how each rule held"
      (let [l (sandbox/line sb "harbour" id first-id)]
        (is (= first-id (:id l)))
        (is (map? (get-in l [:requests :rules :state])))
        (is (seq (:explained l)))
        (is (= (get-in (some #(when (= first-id (:id %)) %) lines) [:decisions :choice])
               (select-keys (get-in l [:decisions :choice]) [:code :confidence])))))
    (testing "what is not there is not found"
      (is (= :ownpurse/not-found (:type (ex-data (try (sandbox/run-lines sb "harbour" "harbour-nope") (catch Exception e e))))))
      (is (= :ownpurse/not-found (:type (ex-data (try (sandbox/run-lines sb "harbour" "../../deps") (catch Exception e e))))))
      (is (= :ownpurse/not-found (:type (ex-data (try (sandbox/world sb "nowhere") (catch Exception e e))))))
      (is (= :ownpurse/not-found (:type (ex-data (try (sandbox/line sb "harbour" id "no-such-line") (catch Exception e e)))))))))

(deftest workspace-scenarios-test
  (let [tmp (.toFile (Files/createTempDirectory "ownpurse-sandbox" (make-array FileAttribute 0)))]
    (try
      (spit (io/file tmp "mine.edn") (slurp (io/resource "classify/eval/harbour.edn")))
      (let [sb (sandbox/open {:dir (str tmp) :scenarios (str tmp)})]
        (testing "a workspace's own scenarios are listed beside ownpurse's, and read from its folder"
          (is (= #{"atlas" "harbour" "mine"} (set (map :name (sandbox/scenarios sb)))))
          (is (= [] (:runs (some #(when (= "mine" (:name %)) %) (sandbox/scenarios sb)))))
          (is (= (:rules (sandbox/world sb "harbour")) (:rules (sandbox/world sb "mine"))))))
      (finally (run! io/delete-file (reverse (file-seq tmp)))))))
