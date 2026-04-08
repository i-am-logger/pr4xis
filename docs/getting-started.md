# Getting Started

## What is Praxis?

Praxis is a Rust framework for building domains where rules matter. You define your state, your actions, and the rules that govern them. The engine enforces those rules at runtime, blocks illegal transitions, and keeps a full trace of everything that happened. Think of it as a state machine with built-in auditing and undo/redo.

## Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)
- `cargo` (comes with Rust)

## Using an Existing Domain

Praxis ships with domains for chess, traffic lights, elevators, music theory, and more. Here is how to use the traffic domain:

Add the dependency:

```toml
[dependencies]
praxis-domains = { git = "https://github.com/i-am-logger/praxis" }
```

Use it:

```rust
use praxis_domains::systems::transportation::traffic::{new_intersection, TrafficAction};

fn main() {
    // Create a 4-way intersection (green=5, yellow=2, red=3 ticks)
    let engine = new_intersection(5, 2, 3);

    // Wait for minimum red duration, then advance North signal to green
    let engine = engine.next(TrafficAction::Tick).unwrap();
    let engine = engine.next(TrafficAction::Tick).unwrap();
    let engine = engine.next(TrafficAction::Tick).unwrap();
    let engine = engine.next(TrafficAction::AdvanceSignal { direction: 0 }).unwrap();

    println!("{}", engine.situation().describe());
    println!("{}", engine.trace().dump());
}
```

The engine enforces that conflicting directions (North/South vs East/West) can never both be green at the same time. If you try, the action is blocked and you get a diagnostic explaining why.

## Building Your Own Domain

Let's build a vending machine. It accepts coins, tracks a balance, and dispenses items when you have enough money.

Create a new project:

```bash
cargo init vending-machine
cd vending-machine
```

Add the engine crate:

```toml
[dependencies]
praxis = { git = "https://github.com/i-am-logger/praxis" }
```

### Step 1: Define Your State

A `Situation` is an immutable snapshot of the world. Implement the `Situation` trait for your state type.

```rust
use praxis::engine::{Situation, Action, Precondition, PreconditionResult, Engine};

#[derive(Debug, Clone, PartialEq)]
struct VendingMachine {
    balance_cents: u32,
    items: Vec<(String, u32)>,  // (name, price in cents)
    dispensed: Vec<String>,
}

impl Situation for VendingMachine {
    fn describe(&self) -> String {
        format!(
            "balance={}c, {} items available, {} dispensed",
            self.balance_cents,
            self.items.len(),
            self.dispensed.len(),
        )
    }

    fn is_terminal(&self) -> bool {
        self.items.is_empty() // no items left to sell
    }
}
```

`describe()` returns a human-readable summary for traces. `is_terminal()` says whether the system has reached a final state.

### Step 2: Define Your Actions

An `Action` is something that can happen. It does not apply itself -- the engine does that after checking preconditions.

```rust
#[derive(Debug, Clone, PartialEq)]
enum VendingAction {
    InsertCoin { cents: u32 },
    SelectItem { name: String },
    ReturnCoins,
}

impl Action for VendingAction {
    type Sit = VendingMachine;

    fn describe(&self) -> String {
        match self {
            VendingAction::InsertCoin { cents } => format!("insert {}c", cents),
            VendingAction::SelectItem { name } => format!("select '{}'", name),
            VendingAction::ReturnCoins => "return coins".to_string(),
        }
    }
}
```

The `type Sit` associated type ties this action to its situation. Every action knows what kind of state it operates on.

### Step 3: Define Your Rules

A `Precondition` checks whether an action is allowed in the current situation. It returns `Satisfied` or `Violated`, each carrying a rule name and explanation.

```rust
struct SufficientFunds;

impl Precondition<VendingAction> for SufficientFunds {
    fn check(&self, machine: &VendingMachine, action: &VendingAction) -> PreconditionResult {
        if let VendingAction::SelectItem { name } = action {
            // Find the item and check if we can afford it
            if let Some((_, price)) = machine.items.iter().find(|(n, _)| n == name) {
                if machine.balance_cents < *price {
                    return PreconditionResult::violated(
                        "sufficient_funds",
                        &format!("need {}c but only have {}c", price, machine.balance_cents),
                        &machine.describe(),
                        &action.describe(),
                    );
                }
            }
        }
        PreconditionResult::satisfied("sufficient_funds", "balance covers cost")
    }

    fn describe(&self) -> &str {
        "balance must be >= item price"
    }
}

struct ItemExists;

impl Precondition<VendingAction> for ItemExists {
    fn check(&self, machine: &VendingMachine, action: &VendingAction) -> PreconditionResult {
        if let VendingAction::SelectItem { name } = action {
            if !machine.items.iter().any(|(n, _)| n == name) {
                return PreconditionResult::violated(
                    "item_exists",
                    &format!("'{}' is not available", name),
                    &machine.describe(),
                    &action.describe(),
                );
            }
        }
        PreconditionResult::satisfied("item_exists", "item is in stock")
    }

    fn describe(&self) -> &str {
        "selected item must exist in the machine"
    }
}

struct ValidCoin;

impl Precondition<VendingAction> for ValidCoin {
    fn check(&self, machine: &VendingMachine, action: &VendingAction) -> PreconditionResult {
        if let VendingAction::InsertCoin { cents } = action {
            let valid = [5, 10, 25, 100]; // nickel, dime, quarter, dollar
            if !valid.contains(cents) {
                return PreconditionResult::violated(
                    "valid_coin",
                    &format!("{}c is not an accepted denomination", cents),
                    &machine.describe(),
                    &action.describe(),
                );
            }
        }
        PreconditionResult::satisfied("valid_coin", "coin is accepted")
    }

    fn describe(&self) -> &str {
        "only 5c, 10c, 25c, and 100c coins are accepted"
    }
}
```

Each precondition checks one rule. The engine runs all of them on every action. If any fail, the action is blocked and the violations are returned with full context.

### Step 4: Create the Engine and Run It

The engine needs three things: an initial situation, preconditions, and an apply function that produces the next situation.

```rust
fn apply(machine: &VendingMachine, action: &VendingAction) -> VendingMachine {
    let mut next = machine.clone();
    match action {
        VendingAction::InsertCoin { cents } => {
            next.balance_cents += cents;
        }
        VendingAction::SelectItem { name } => {
            if let Some(pos) = next.items.iter().position(|(n, _)| n == name) {
                let (item_name, price) = next.items.remove(pos);
                next.balance_cents -= price;
                next.dispensed.push(item_name);
            }
        }
        VendingAction::ReturnCoins => {
            next.balance_cents = 0;
        }
    }
    next
}

fn main() {
    let machine = VendingMachine {
        balance_cents: 0,
        items: vec![
            ("Soda".into(), 125),
            ("Chips".into(), 100),
            ("Candy".into(), 75),
        ],
        dispensed: vec![],
    };

    let engine = Engine::new(
        machine,
        vec![
            Box::new(SufficientFunds),
            Box::new(ItemExists),
            Box::new(ValidCoin),
        ],
        apply,
    );

    // Insert some coins
    let engine = engine.next(VendingAction::InsertCoin { cents: 100 }).unwrap();
    let engine = engine.next(VendingAction::InsertCoin { cents: 25 }).unwrap();

    // Buy a soda (costs 125c, we have exactly 125c)
    let engine = engine.next(VendingAction::SelectItem { name: "Soda".into() }).unwrap();

    println!("Current state: {}", engine.situation().describe());
    // => "balance=0c, 2 items available, 1 dispensed"
}
```

### Step 5: Check the Trace

Every action is recorded, whether it succeeded or not. The trace tells you exactly what happened and why.

```rust
    // Print the full trace
    println!("{}", engine.trace().dump());
    // [OK] insert 100c | balance=0c, 3 items available, 0 dispensed -> balance=100c, 3 items available, 0 dispensed
    //   + sufficient_funds: balance covers cost
    //   + item_exists: item is in stock
    //   + valid_coin: coin is accepted
    // [OK] insert 25c | balance=100c, 3 items available, 0 dispensed -> balance=125c, 3 items available, 0 dispensed
    //   ...
    // [OK] select 'Soda' | balance=125c, 3 items available, 0 dispensed -> balance=0c, 2 items available, 1 dispensed
    //   ...

    // You can also inspect programmatically
    assert_eq!(engine.trace().successful_steps(), 3);
    assert_eq!(engine.trace().violations(), 0);
```

Violations are also recorded. If you try to buy something you can't afford:

```rust
    // Try to buy chips with no money
    let result = engine.next(VendingAction::SelectItem { name: "Chips".into() });
    match result {
        Ok(_) => println!("bought it"),
        Err((engine_back, violations)) => {
            println!("blocked! {} violations:", violations.len());
            for v in &violations {
                println!("  {}: {}", v.rule(), v.reason());
            }
            // => "blocked! 1 violations:"
            // => "  sufficient_funds: need 100c but only have 0c"

            // The engine is returned so you can keep going
            let _engine = engine_back;
        }
    }
```

### Step 6: Try Undo/Redo

The engine keeps a full history. `back()` undoes the last action, `forward()` redoes it.

```rust
    // Starting from the state after buying soda (balance=0, 2 items, 1 dispensed)
    let engine = engine.back().unwrap();
    println!("{}", engine.situation().describe());
    // => "balance=125c, 3 items available, 0 dispensed"
    // We're back to before we bought the soda

    let engine = engine.back().unwrap();
    println!("{}", engine.situation().describe());
    // => "balance=100c, 3 items available, 0 dispensed"
    // Back to before the second coin

    // Redo
    let engine = engine.forward().unwrap();
    println!("{}", engine.situation().describe());
    // => "balance=125c, 3 items available, 0 dispensed"

    // You can check how deep the history goes
    println!("undo depth: {}", engine.back_depth());    // 1
    println!("redo depth: {}", engine.forward_depth());  // 1

    // Taking a new action clears the redo stack
    let engine = engine.next(VendingAction::SelectItem { name: "Candy".into() }).unwrap();
    println!("redo depth: {}", engine.forward_depth());  // 0
```

`back()` and `forward()` return `Err(self)` if there's nowhere to go (at the beginning or end of history), so you can handle that however you want.

## Adding Ontology

The engine handles runtime enforcement. If you also want to formally describe your domain's structure -- what things exist, how they relate, and what invariants hold -- you can layer in the ontology modules.

This is optional. Many domains work fine with just the engine. Ontology is useful when you want to verify structural properties at compile time or via tests, separate from runtime behavior.

### Entity

An `Entity` is something that exists in your domain. It must be finite and enumerable.

```rust
use praxis::category::Entity;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ItemType { Soda, Chips, Candy }

impl Entity for ItemType {
    fn variants() -> Vec<Self> {
        vec![ItemType::Soda, ItemType::Chips, ItemType::Candy]
    }
}
```

### Relationship

A `Relationship` is a directed connection between entities. It forms the morphisms in your category.

```rust
use praxis::category::Relationship;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheaperThan {
    from: ItemType,
    to: ItemType,
}

impl Relationship for CheaperThan {
    type Object = ItemType;
    fn source(&self) -> ItemType { self.from.clone() }
    fn target(&self) -> ItemType { self.to.clone() }
}
```

### Category

A `Category` ties entities and relationships together with identity and composition laws.

```rust
use praxis::category::Category;

struct ItemCategory;

impl Category for ItemCategory {
    type Object = ItemType;
    type Morphism = CheaperThan;

    fn identity(obj: &ItemType) -> CheaperThan {
        CheaperThan { from: obj.clone(), to: obj.clone() }
    }

    fn compose(f: &CheaperThan, g: &CheaperThan) -> Option<CheaperThan> {
        if f.to == g.from {
            Some(CheaperThan { from: f.from.clone(), to: g.to.clone() })
        } else {
            None
        }
    }

    fn morphisms() -> Vec<CheaperThan> {
        // Candy(75) < Chips(100) < Soda(125)
        vec![
            Self::identity(&ItemType::Soda),
            Self::identity(&ItemType::Chips),
            Self::identity(&ItemType::Candy),
            CheaperThan { from: ItemType::Candy, to: ItemType::Chips },
            CheaperThan { from: ItemType::Chips, to: ItemType::Soda },
            CheaperThan { from: ItemType::Candy, to: ItemType::Soda },
        ]
    }
}
```

### Quality

A `Quality` is a property that an entity can have.

```rust
use praxis::ontology::Quality;

#[derive(Debug, Clone)]
struct PriceInCents;

impl Quality for PriceInCents {
    type Individual = ItemType;
    type Value = u32;

    fn get(&self, item: &ItemType) -> Option<u32> {
        match item {
            ItemType::Soda => Some(125),
            ItemType::Chips => Some(100),
            ItemType::Candy => Some(75),
        }
    }
}
```

### Axiom

An `Axiom` is a domain invariant that must always hold.

```rust
use praxis::ontology::Axiom;

struct AllItemsHavePrices;

impl Axiom<ItemCategory> for AllItemsHavePrices {
    fn description(&self) -> &str {
        "every item must have a price"
    }

    fn holds(&self) -> bool {
        ItemType::variants().iter().all(|item| PriceInCents.get(item).is_some())
    }
}
```

### Validation

You can verify category laws and axioms in tests:

```rust
#[test]
fn test_category_laws() {
    praxis::category::validate::check_category_laws::<ItemCategory>().unwrap();
}

#[test]
fn test_all_items_priced() {
    assert!(AllItemsHavePrices.holds());
}
```

## Next Steps

- [architecture.md](architecture.md) -- the three-layer design and how the modules compose
- [concepts.md](concepts.md) -- deeper explanation of situations, actions, preconditions, and traces
- [domain-crates.md](domain-crates.md) -- reference for all built-in domains
