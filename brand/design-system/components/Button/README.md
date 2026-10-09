# Button

Buttons: primary in ink, one coin button for the action a view exists for, quiet for the rest. - `primary` (ink fill) for the main action in a panel. `coin` at most once per view, for the thing the page is for (Post, Reconcile). `quiet` (outlined, the default) for everything else. `ghost` for inline actions in lapis.
- Labels are verbs in sentence case: "Post entry", "Sync now", "Show differences". Never "Submit", "OK" or "Click here".
- A destructive or irreversible write says what it does: "Void transaction", never "Confirm".
- Focus ring: 2px solid `focus`, 2px offset.
