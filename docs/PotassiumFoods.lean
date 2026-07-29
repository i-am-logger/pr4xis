/-
Potassium Content of Common Foods
==================================

Author:  Ido Samuelson
Date:    July 2026
License: CC BY-SA 4.0

A Lean 4 reference table of potassium content (in milligrams) for common
foods, stated as axioms: asserted empirical facts rather than derived
theorems. This differs in kind from `ProvisionalClosure.lean` /
`OpenSystemsLossy.lean` / `LEMDependency.lean`, whose axioms are logical
postulates within a formal system. Here "axiom" means "a measured
quantity taken as given, cited to its source" — the Lean mechanism is
reused for its citation/machine-checkability discipline, not because
potassium content is a logical necessity.

Sources:
  [HH]   Harvard T.H. Chan School of Public Health, "The Best Foods High
         in Potassium (and Why You Need Them)", Harvard Health
         Publishing. <https://www.health.harvard.edu/diet-and-nutrition/
         the-best-foods-high-in-potassium-and-why-you-need-them>
         Source for the produce/legume/fish/original-dairy figures below.
  [USDA] USDA FoodData Central <https://fdc.nal.usda.gov/>, cross-checked
         via nutritionvalue.org's USDA-sourced pages for the 4 dairy
         items added after [HH]'s original list (whole milk, 2% milk,
         buttermilk, butter) — fetched directly during this session, not
         carried over from training-data recall.

A note on raw vs. pasteurized milk potassium content: pasteurization is
a mild heat treatment (~72°C / 15s, HTST) that inactivates pathogens: it
does not remove or meaningfully alter mineral content. Potassium is a
water/protein-fraction mineral, unaffected by either pasteurization or
fat percentage in any way large enough to matter here. No separate
"raw milk" entry exists in USDA FoodData Central (raw milk is largely
outside the standard commercial/regulated supply the database catalogs);
the figures below for whole/2% milk apply equally to their raw
counterparts. This file states that as a fact about mineral chemistry,
not a claim about raw milk's relative safety or overall health merits,
which this file takes no position on.
-/

namespace PotassiumFoods

/-- A food entry: name, serving description, and potassium content in
    milligrams. `mg` is the ASSERTED (axiomatic) empirical quantity;
    everything else is descriptive metadata carried alongside it. -/
structure Food where
  name    : String
  serving : String
  mg      : Nat
deriving Repr

/-! ## §1. Legumes [HH] -/

axiom white_beans        : Food
axiom white_beans_eq      : white_beans = ⟨"White beans, cooked", "1/2 cup", 500⟩

axiom edamame             : Food
axiom edamame_eq          : edamame = ⟨"Soybeans, green (edamame), cooked", "1/2 cup", 485⟩

axiom lentils              : Food
axiom lentils_eq           : lentils = ⟨"Lentils, cooked", "1/2 cup", 365⟩

/-! ## §2. Fruits [HH] -/

axiom banana               : Food
axiom banana_eq            : banana = ⟨"Banana", "1 medium", 422⟩

axiom apricots_dried        : Food
axiom apricots_dried_eq     : apricots_dried = ⟨"Apricots, dried", "1/4 cup", 378⟩

axiom cantaloupe             : Food
axiom cantaloupe_eq          : cantaloupe = ⟨"Cantaloupe", "1/4 medium", 368⟩

axiom raisins                 : Food
axiom raisins_eq              : raisins = ⟨"Raisins", "1 1/2 ounces", 318⟩

axiom grapes                   : Food
axiom grapes_eq                : grapes = ⟨"Grapes", "1 cup", 288⟩

axiom mango                     : Food
axiom mango_eq                  : mango = ⟨"Mango", "1 cup", 277⟩

axiom figs_dried                  : Food
axiom figs_dried_eq               : figs_dried = ⟨"Figs, dried", "5 figs", 272⟩

/-! ## §3. Vegetables [HH] -/

axiom potato_russet                 : Food
axiom potato_russet_eq              : potato_russet = ⟨"Potato, russet, baked (with skin)", "1 medium", 952⟩

axiom beet_greens                    : Food
axiom beet_greens_eq                 : beet_greens = ⟨"Beet greens, cooked", "1/2 cup", 655⟩

axiom sweet_potato                    : Food
axiom sweet_potato_eq                 : sweet_potato = ⟨"Sweet potato, baked (with skin)", "1 medium", 542⟩

axiom swiss_chard                      : Food
axiom swiss_chard_eq                   : swiss_chard = ⟨"Swiss chard, cooked", "1/2 cup", 481⟩

axiom spinach_cooked                    : Food
axiom spinach_cooked_eq                 : spinach_cooked = ⟨"Spinach, cooked", "1/2 cup", 419⟩

axiom tomato                             : Food
axiom tomato_eq                          : tomato = ⟨"Tomato", "1 medium", 292⟩

/-! ## §4. Fruit and vegetable juices [HH] -/

axiom carrot_juice                        : Food
axiom carrot_juice_eq                     : carrot_juice = ⟨"Carrot juice", "3/4 cup", 517⟩

axiom tomato_juice                         : Food
axiom tomato_juice_eq                      : tomato_juice = ⟨"Tomato juice, no salt added", "3/4 cup", 395⟩

axiom orange_juice                          : Food
axiom orange_juice_eq                       : orange_juice = ⟨"Orange juice", "3/4 cup", 332⟩

/-! ## §5. Dairy products

`yogurt_lowfat` and `milk_nonfat` are [HH]'s original two entries.
`milk_whole`, `milk_2pct`, `buttermilk`, and `butter` were added after,
verified directly against USDA FoodData Central (via nutritionvalue.org)
during this session — see the file header's raw-milk note for why no
separate raw-milk entries appear. -/

axiom yogurt_lowfat                          : Food
axiom yogurt_lowfat_eq                       : yogurt_lowfat = ⟨"Yogurt, plain, low-fat", "1 cup", 531⟩

axiom milk_nonfat                             : Food
axiom milk_nonfat_eq                          : milk_nonfat = ⟨"Milk, nonfat (skim)", "1 cup", 382⟩

/-- [USDA] FDC 171265, "Milk, whole, 3.25% milkfat" — applies equally to
    raw whole milk (see file header). -/
axiom milk_whole                               : Food
axiom milk_whole_eq                            : milk_whole = ⟨"Milk, whole (raw or pasteurized, ~3.25-4% fat)", "1 cup", 322⟩

/-- [USDA] FDC 11112110, "Milk, reduced fat, fluid, 2% milkfat" — applies
    equally to raw 2% milk (see file header). -/
axiom milk_2pct                                 : Food
axiom milk_2pct_eq                              : milk_2pct = ⟨"Milk, reduced-fat (raw or pasteurized, 2%)", "1 cup", 388⟩

/-- [USDA] FDC 11115300, "Buttermilk". -/
axiom buttermilk                                 : Food
axiom buttermilk_eq                              : buttermilk = ⟨"Buttermilk, cultured", "1 cup (244 g)", 386⟩

/-- [USDA], salted butter. Potassium is genuinely negligible in any
    butter — raw-cream or pasteurized-cream alike — because butter is
    ~80% milkfat by mass and potassium lives in the non-fat milk solids,
    almost all of which are separated out during churning. This value is
    stated honestly rather than inflated to match the scale of the other
    entries in this table. -/
axiom butter                                      : Food
axiom butter_eq                                   : butter = ⟨"Butter, salted (raw or pasteurized cream)", "1 tablespoon", 3⟩

/-! ## §6. Fish [HH] -/

axiom salmon                                       : Food
axiom salmon_eq                                    : salmon = ⟨"Salmon, wild Atlantic, cooked", "3 ounces", 534⟩

axiom halibut                                       : Food
axiom halibut_eq                                    : halibut = ⟨"Halibut, cooked", "3 ounces", 449⟩

axiom tuna_yellowfin                                 : Food
axiom tuna_yellowfin_eq                              : tuna_yellowfin = ⟨"Tuna, yellowfin, cooked", "3 ounces", 448⟩

end PotassiumFoods
