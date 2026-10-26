(ns ownpurse.export.xlsx
  "A minimal XLSX writer: one sheet per table, a header row in bold, amounts as numbers written as their exact
  decimal text (never a double, never a formula), everything else as text."
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str])
  (:import
   (java.io OutputStream)
   (java.nio.charset StandardCharsets)
   (java.util.zip ZipEntry ZipOutputStream)))

(defn- xml-escape [s]
  (-> (str s)
      (str/replace "&" "&amp;") (str/replace "<" "&lt;") (str/replace ">" "&gt;") (str/replace "\"" "&quot;")
      ;; XML 1.0 has no place for control characters other than tab, newline and return.
      (str/replace #"[\x00-\x08\x0B\x0C\x0E-\x1F]" "")))

(defn- column-name [i]
  (loop [i (inc i) s ""]
    (if (zero? i) s (recur (quot (dec i) 26) (str (char (+ 65 (rem (dec i) 26))) s)))))

(defn- cell [ref v style]
  (if (decimal? v)
    (str "<c r=\"" ref "\"" style "><v>" (.toPlainString ^BigDecimal v) "</v></c>")
    (str "<c r=\"" ref "\" t=\"inlineStr\"" style "><is><t xml:space=\"preserve\">" (xml-escape v) "</t></is></c>")))

(defn- sheet-xml [{:keys [columns rows]}]
  (let [row (fn [r cells style]
              (str "<row r=\"" (inc r) "\">"
                   (apply str (map-indexed (fn [c v] (cell (str (column-name c) (inc r)) v style)) cells))
                   "</row>"))]
    (str "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
         "<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">"
         "<sheetViews><sheetView workbookViewId=\"0\"><pane ySplit=\"1\" topLeftCell=\"A2\" state=\"frozen\"/>"
         "</sheetView></sheetViews><sheetData>"
         (row 0 columns " s=\"1\"")
         (apply str (map-indexed (fn [i cells] (row (inc i) cells "")) rows))
         "</sheetData></worksheet>")))

(defn sheet-name
  "A sheet's name as Excel allows it: at most 31 characters, none of []:*?/\\."
  [s]
  (let [s (str/replace (str s) #"[\[\]:*?/\\]" "-")]
    (subs s 0 (min 31 (count s)))))

(defn- workbook-parts [sheets]
  (let [n (count sheets)]
    (concat
     [["[Content_Types].xml"
       (str "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
            "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">"
            "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>"
            "<Default Extension=\"xml\" ContentType=\"application/xml\"/>"
            "<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>"
            "<Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>"
            (apply str (for [i (range 1 (inc n))]
                         (str "<Override PartName=\"/xl/worksheets/sheet" i ".xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>")))
            "</Types>")]
      ["_rels/.rels"
       (str "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">"
            "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>"
            "</Relationships>")]
      ["xl/workbook.xml"
       (str "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
            "<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" "
            "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets>"
            (apply str (map-indexed (fn [i [name]]
                                      (str "<sheet name=\"" (xml-escape (sheet-name name)) "\" sheetId=\"" (inc i)
                                           "\" r:id=\"rId" (inc i) "\"/>"))
                                    sheets))
            "</sheets></workbook>")]
      ["xl/_rels/workbook.xml.rels"
       (str "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">"
            (apply str (for [i (range 1 (inc n))]
                         (str "<Relationship Id=\"rId" i "\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet" i ".xml\"/>")))
            "<Relationship Id=\"rId" (inc n) "\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>"
            "</Relationships>")]
      ["xl/styles.xml"
       (str "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
            "<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">"
            "<fonts count=\"2\"><font><sz val=\"11\"/><name val=\"Calibri\"/></font>"
            "<font><b/><sz val=\"11\"/><name val=\"Calibri\"/></font></fonts>"
            "<fills count=\"2\"><fill><patternFill patternType=\"none\"/></fill><fill><patternFill patternType=\"gray125\"/></fill></fills>"
            "<borders count=\"1\"><border><left/><right/><top/><bottom/><diagonal/></border></borders>"
            "<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>"
            "<cellXfs count=\"2\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>"
            "<xf numFmtId=\"0\" fontId=\"1\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\"/></cellXfs>"
            "<cellStyles count=\"1\"><cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/></cellStyles>"
            "</styleSheet>")]]
     (map-indexed (fn [i [_ table]] [(str "xl/worksheets/sheet" (inc i) ".xml") (sheet-xml table)]) sheets))))

(defn write!
  "Writes `sheets`, [[name {:columns [...] :rows [[cell ...] ...]}] ...], as an XLSX file at `file`. A BigDecimal
  cell is a number; any other cell is text."
  [file sheets]
  (with-open [zip (ZipOutputStream. (io/output-stream (io/file file)))]
    (doseq [[path content] (workbook-parts sheets)]
      (.putNextEntry zip (ZipEntry. ^String path))
      (.write ^OutputStream zip (.getBytes ^String content StandardCharsets/UTF_8))
      (.closeEntry zip))))
