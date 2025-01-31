;; This Source Code Form is subject to the terms of the Mozilla Public
;; License, v. 2.0. If a copy of the MPL was not distributed with this
;; file, You can obtain one at http://mozilla.org/MPL/2.0/.
;;
;; Copyright (c) KALEIDOS INC

(ns app.main.data.workspace.variations
  (:require
   [app.common.colors :as clr]
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


#_(defn add-variation2
  [id]
  (ptk/reify ::add-variation
    ptk/WatchEvent
    (watch [it state _]
      (let [variation-id (uuid/next)
            page-id      (:current-page-id state)
            objects      (dsh/lookup-page-objects state page-id)
            main         (get objects id)
            orig-comp-id (:component-id main)

            file-id       (:current-file-id state)
            library       (-> (dsh/lookup-libraries state)
                              (get file-id))
            component    (ctkl/get-component (:data library) orig-comp-id)


            {:keys [changes new-main-instance-id new-component-id new-shapes]}
            (-> (pcb/empty-changes it page-id)
                (pcb/with-library-data (:data library))
                (cll/prepare-duplicate-component library orig-comp-id true))



            all-objects (merge objects (reduce (fn [acc item]
                                                 (assoc acc (:id item) item))
                                               {}
                                               new-shapes))

            ;;TODO This line proce the error No protocol method IReversible.-rseq defined for type cljs.core/Cons
            ;;all-objects (update-in all-objects [(:parent-id main) :shapes] #(cons new-main-instance-id %))

            ;; So we fo it in two steps
            shapes (into [new-main-instance-id] (get-in all-objects [(:parent-id main) :shapes]))
            all-objects (assoc-in all-objects [(:parent-id main) :shapes] shapes)


            [frame-shape changes]
            (cfsh/prepare-create-artboard-from-selection changes
                                                         variation-id
                                                         nil
                                                         all-objects
                                                         #{new-main-instance-id id}
                                                         0
                                                         (:name main)
                                                         false)

            ;; TODO add flex and strokes to the board


            new-component (assoc component :id new-component-id :main-instance-id new-main-instance-id)
            changes (-> changes
                        (pcb/update-component orig-comp-id #(assoc % :variation-id variation-id
                                                                   :variation-properties {:property1 "value1"}))
                        (pcb/update-component new-component-id new-component #(assoc % :variation-id variation-id
                                                                                     :variation-properties {:property1 "value2"})))


            undo-id  (js/Symbol)]

        (when changes
          (rx/of
           (dwu/start-undo-transaction undo-id)
           (dch/commit-changes changes)
           (dws/select-shapes (d/ordered-set (:id frame-shape)))
           (ptk/data-event :layout/update {:ids [(:id frame-shape)]})
           (dwu/commit-undo-transaction undo-id)))))))


(defn update-property-value
  [component-id key value]
  (ptk/reify ::update-property-value
    ptk/WatchEvent
    (watch [it state _]
      (let [page-id      (:current-page-id state)
            file-id       (:current-file-id state)
            library       (-> (dsh/lookup-libraries state)
                              (get file-id))
            changes (-> (pcb/empty-changes it page-id)
                        (pcb/with-library-data (:data library))
                        (pcb/update-component component-id #(assoc-in % [:variation-properties key] value)))
            undo-id  (js/Symbol)]
        (rx/of
         (dwu/start-undo-transaction undo-id)
         (dch/commit-changes changes)
         (dwu/commit-undo-transaction undo-id))))))

(defn update-variation
  [component-id variation-id]
  (ptk/reify ::update-property-value
    ptk/WatchEvent
    (watch [it state _]
      (let [page-id      (:current-page-id state)
            file-id       (:current-file-id state)
            library       (-> (dsh/lookup-libraries state)
                              (get file-id))
            changes (-> (pcb/empty-changes it page-id)
                        (pcb/with-library-data (:data library))
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
                                                    :is-variation true))
         (dwsh/update-shapes [main-id] #(assoc % :layout-item-h-sizing :fix :layout-item-v-sizing :fix))
         (cl/add-stroke [variation-id] {:stroke-alignment :inner
                                        :stroke-style :solid
                                        :stroke-color "#bb97d8" ;; todo use color var?
                                        :stroke-opacity 1
                                        :stroke-width 2})
         (dwl/duplicate-component file-id (:component-id main) new-component-id)
         (update-variation (:component-id main) variation-id)
         (update-property-value (:component-id main) :property1 "value1")
         (update-variation new-component-id variation-id)
         (update-property-value new-component-id :property1 "value2")
         (dwu/commit-undo-transaction undo-id))))))



