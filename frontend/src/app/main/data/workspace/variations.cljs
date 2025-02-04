;; This Source Code Form is subject to the terms of the Mozilla Public
;; License, v. 2.0. If a copy of the MPL was not distributed with this
;; file, You can obtain one at http://mozilla.org/MPL/2.0/.
;;
;; Copyright (c) KALEIDOS INC

(ns app.main.data.workspace.variations
  (:require
   [app.common.colors :as clr]
   [app.common.data.macros :as dm]
   [app.common.files.changes-builder :as pcb]
   [app.common.uuid :as uuid]
   [app.main.data.changes :as dch]
   [app.main.data.helpers :as dsh]
   [app.main.data.workspace.colors :as cl]
   [app.main.data.workspace.libraries :as dwl]
   [app.main.data.workspace.shape-layout :as dwsl]
   [app.main.data.workspace.shapes :as dwsh]
   [app.main.data.workspace.undo :as dwu]
   [beicon.v2.core :as rx]
   [potok.v2.core :as ptk]))


(defn update-property-name
  "Update the property name on all the components with this variation-id"
  [variation-id pos new-name]
  (ptk/reify ::update-property-name
    ptk/WatchEvent
    (watch [it state _]
      (let [page-id (:current-page-id state)
            data    (dsh/lookup-file-data state)

            related-components (->> (:components data)
                                    vals
                                    (filter #(= (:variation-id %) variation-id)))

            changes (-> (pcb/empty-changes it page-id)
                        (pcb/with-library-data data))

            changes (reduce (fn [changes component]
                              (pcb/update-component changes (:id component) #(assoc-in % [:variation-properties pos :name] new-name)))
                            changes
                            related-components)
            undo-id (js/Symbol)]
        (rx/of
         (dwu/start-undo-transaction undo-id)
         (dch/commit-changes changes)
         (dwu/commit-undo-transaction undo-id))))))


(defn update-property-value
  [component-id pos name value]
  (ptk/reify ::update-property-value
    ptk/WatchEvent
    (watch [it state _]
      (let [page-id (:current-page-id state)
            data    (dsh/lookup-file-data state)
            changes (-> (pcb/empty-changes it page-id)
                        (pcb/with-library-data data)
                        (pcb/update-component component-id #(assoc-in % [:variation-properties pos :value] value)))
            undo-id  (js/Symbol)]
        (rx/of
         (dwu/start-undo-transaction undo-id)
         (dch/commit-changes changes)
         (dwu/commit-undo-transaction undo-id))))))

(defn add-property
  [component-id name value]
  (ptk/reify ::add-property
    ptk/WatchEvent
    (watch [it state _]
      (let [page-id    (:current-page-id state)
            data       (dsh/lookup-file-data state)
            component  (dm/get-in data [:components component-id])
            properties (-> (or (:variation-properties component) [])
                           (conj {:name name :value value}))
            changes    (-> (pcb/empty-changes it page-id)
                           (pcb/with-library-data data)
                           (pcb/update-component component-id #(assoc % :variation-properties properties)))
            undo-id  (js/Symbol)]
        (rx/of
         (dwu/start-undo-transaction undo-id)
         (dch/commit-changes changes)
         (dwu/commit-undo-transaction undo-id))))))

(defn update-variation
  [component-id variation-id]
  (ptk/reify ::update-variation
    ptk/WatchEvent
    (watch [it state _]
      (let [page-id      (:current-page-id state)
            data    (dsh/lookup-file-data state)
            changes (-> (pcb/empty-changes it page-id)
                        (pcb/with-library-data data)
                        (pcb/update-component component-id #(assoc % :variation-id variation-id)))
            undo-id  (js/Symbol)]
        (rx/of
         (dwu/start-undo-transaction undo-id)
         (dch/commit-changes changes)
         (dwu/commit-undo-transaction undo-id))))))

(defn add-variation
  [id]
  (ptk/reify ::add-variation
    ptk/WatchEvent
    (watch [it state _]
      (let [variation-id (uuid/next)
            new-component-id (uuid/next)
            file-id      (:current-file-id state)
            page-id      (:current-page-id state)
            objects      (dsh/lookup-page-objects state page-id)
            main         (get objects id)
            main-id      (:id main)
            undo-id  (js/Symbol)]


        (rx/of
         (dwu/start-undo-transaction undo-id)
         (dwsh/create-artboard-from-selection variation-id)
         (cl/remove-all-fills [variation-id] {:color clr/black :opacity 1})
         (dwsl/create-layout-from-id variation-id :flex)
         (dwsh/update-shapes [variation-id] #(assoc % :layout-item-h-sizing :auto
                                                    :layout-item-v-sizing :auto
                                                    :layout-padding {:p1 30 :p2 30 :p3 30 :p4 30}
                                                    :layout-gap     {:row-gap 0 :column-gap 20}
                                                    :name (:name main)
                                                    :r1 20
                                                    :r2 20
                                                    :r3 20
                                                    :r4 20
                                                    :is-variation-container true))
         (dwsh/update-shapes [main-id] #(assoc % :layout-item-h-sizing :fix :layout-item-v-sizing :fix))
         (cl/add-stroke [variation-id] {:stroke-alignment :inner
                                        :stroke-style :solid
                                        :stroke-color "#bb97d8" ;; todo use color var?
                                        :stroke-opacity 1
                                        :stroke-width 2})
         (dwl/duplicate-component file-id (:component-id main) new-component-id)
         (update-variation (:component-id main) variation-id)
         (add-property (:component-id main) "Property 1" "Value1")
         (update-variation new-component-id variation-id)
         (add-property new-component-id "Property 1" "Value2")
         (dwu/commit-undo-transaction undo-id))))))



