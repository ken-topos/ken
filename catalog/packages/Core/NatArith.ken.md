# `Nat` arithmetic — canonical operations and free algebraic laws

Natural-number addition and multiplication compute by structural recursion.
Their familiar identity, commutativity, associativity, and distributivity laws
are stated before their induction machinery, so readers meet the algebraic
interface before the proof implementation.

## Operations and laws

The two computational names are introduced first because they occur in every
law's type. The headline laws follow immediately; proofs that need induction
are thin wrappers around recursive helpers collected later in the declaration
group.

```ken
fn add (a : Nat) (b : Nat) : Nat =
  match b {
    Zero => a ;
    Suc b2 => Suc (add a b2)
  }

fn mul (a : Nat) (b : Nat) : Nat =
  match b {
    Zero => Zero ;
    Suc b2 => add (mul a b2) a
  }

lemma add_zero_r (a : Nat) : Equal Nat (add a Zero) a = Refl

fn add_zero_l_ind (a : Nat) : Equal Nat (add Zero a) a =
  match a {
    Zero => tt ;
    Suc a2 => cong Nat Nat (add Zero a2) a2 Suc (add_zero_l_ind a2)
  }

lemma add_zero_l (a : Nat) : Equal Nat (add Zero a) a = add_zero_l_ind a

lemma add_suc_r (a : Nat) (b : Nat)
  : Equal Nat (add a (Suc b)) (Suc (add a b)) = Refl

fn add_suc_l_ind (a : Nat) (b : Nat)
  : Equal Nat (add (Suc a) b) (Suc (add a b)) =
  match b {
    Zero => Refl ;
    Suc b2 =>
      cong Nat Nat (add (Suc a) b2) (Suc (add a b2)) Suc (add_suc_l_ind a b2)
  }

lemma add_suc_l (a : Nat) (b : Nat)
  : Equal Nat (add (Suc a) b) (Suc (add a b)) = add_suc_l_ind a b

fn add_assoc_ind (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (add a (add b c)) (add (add a b) c) =
  match c {
    Zero => Refl ;
    Suc c2 =>
      cong Nat Nat
        (add a (add b c2))
        (add (add a b) c2)
        Suc
        (add_assoc_ind a b c2)
  }

lemma add_assoc (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (add a (add b c)) (add (add a b) c) = add_assoc_ind a b c

fn add_comm_ind (a : Nat) (b : Nat) : Equal Nat (add a b) (add b a) =
  match b {
    Zero => sym Nat (add Zero a) a (add_zero_l a) ;
    Suc b2 =>
      trans Nat
        (add a (Suc b2))
        (Suc (add b2 a))
        (add (Suc b2) a)
        (cong Nat Nat (add a b2) (add b2 a) Suc (add_comm_ind a b2))
        (sym Nat
          (add (Suc b2) a)
          (Suc (add b2 a))
          (add_suc_l b2 a))
  }

lemma add_comm (a : Nat) (b : Nat) : Equal Nat (add a b) (add b a) =
  add_comm_ind a b

lemma mul_zero_r (a : Nat) : Equal Nat (mul a Zero) Zero = tt

fn mul_zero_l_ind (a : Nat) : Equal Nat (mul Zero a) Zero =
  match a {
    Zero => tt ;
    Suc a2 => mul_zero_l_ind a2
  }

lemma mul_zero_l (a : Nat) : Equal Nat (mul Zero a) Zero = mul_zero_l_ind a

lemma mul_suc_r (a : Nat) (b : Nat)
  : Equal Nat (mul a (Suc b)) (add (mul a b) a) = Refl

fn mul_suc_l_ind (a : Nat) (b : Nat)
  : Equal Nat (mul (Suc a) b) (add (mul a b) b) =
  match b {
    Zero => tt ;
    Suc b2 =>
      cong Nat Nat
        (add (mul (Suc a) b2) a)
        (add (add (mul a b2) a) b2)
        Suc
        (trans Nat
          (add (mul (Suc a) b2) a)
          (add (add (mul a b2) b2) a)
          (add (add (mul a b2) a) b2)
          (cong Nat Nat
            (mul (Suc a) b2)
            (add (mul a b2) b2)
            (λx. add x a)
            (mul_suc_l_ind a b2))
          (trans Nat
            (add (add (mul a b2) b2) a)
            (add (mul a b2) (add b2 a))
            (add (add (mul a b2) a) b2)
            (sym Nat
              (add (mul a b2) (add b2 a))
              (add (add (mul a b2) b2) a)
              (add_assoc (mul a b2) b2 a))
            (trans Nat
              (add (mul a b2) (add b2 a))
              (add (mul a b2) (add a b2))
              (add (add (mul a b2) a) b2)
              (cong Nat Nat
                (add b2 a)
                (add a b2)
                (λx. add (mul a b2) x)
                (add_comm b2 a))
              (add_assoc (mul a b2) a b2))))
  }

lemma mul_suc_l (a : Nat) (b : Nat)
  : Equal Nat (mul (Suc a) b) (add (mul a b) b) = mul_suc_l_ind a b

lemma mul_one_r (a : Nat) : Equal Nat (mul a (Suc Zero)) a = add_zero_l a

fn mul_one_l_ind (a : Nat) : Equal Nat (mul (Suc Zero) a) a =
  match a {
    Zero => tt ;
    Suc a2 =>
      trans Nat
        (mul (Suc Zero) (Suc a2))
        (add a2 (Suc Zero))
        (Suc a2)
        (cong Nat Nat
          (mul (Suc Zero) a2)
          a2
          (λx. add x (Suc Zero))
          (mul_one_l_ind a2))
        (add_suc_r a2 Zero)
  }

lemma mul_one_l (a : Nat) : Equal Nat (mul (Suc Zero) a) a = mul_one_l_ind a

fn mul_comm_ind (a : Nat) (b : Nat) : Equal Nat (mul a b) (mul b a) =
  match b {
    Zero => sym Nat (mul Zero a) Zero (mul_zero_l a) ;
    Suc b2 =>
      trans Nat
        (mul a (Suc b2))
        (add (mul b2 a) a)
        (mul (Suc b2) a)
        (cong Nat Nat
          (mul a b2)
          (mul b2 a)
          (λx. add x a)
          (mul_comm_ind a b2))
        (sym Nat
          (mul (Suc b2) a)
          (add (mul b2 a) a)
          (mul_suc_l b2 a))
  }

lemma mul_comm (a : Nat) (b : Nat) : Equal Nat (mul a b) (mul b a) =
  mul_comm_ind a b

fn mul_add_distrib_r_ind (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (mul a (add b c)) (add (mul a b) (mul a c)) =
  match c {
    Zero => Refl ;
    Suc c2 =>
      trans Nat
        (add (mul a (add b c2)) a)
        (add (add (mul a b) (mul a c2)) a)
        (add (mul a b) (add (mul a c2) a))
        (cong Nat Nat
          (mul a (add b c2))
          (add (mul a b) (mul a c2))
          (λx. add x a)
          (mul_add_distrib_r_ind a b c2))
        (sym Nat
          (add (mul a b) (add (mul a c2) a))
          (add (add (mul a b) (mul a c2)) a)
          (add_assoc (mul a b) (mul a c2) a))
  }

lemma mul_add_distrib_r (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (mul a (add b c)) (add (mul a b) (mul a c)) =
  mul_add_distrib_r_ind a b c

lemma mul_add_distrib_l (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (mul (add a b) c) (add (mul a c) (mul b c)) =
  trans Nat
    (mul (add a b) c)
    (mul c (add a b))
    (add (mul a c) (mul b c))
    (mul_comm (add a b) c)
    (trans Nat
      (mul c (add a b))
      (add (mul c a) (mul c b))
      (add (mul a c) (mul b c))
      (mul_add_distrib_r c a b)
      (trans Nat
        (add (mul c a) (mul c b))
        (add (mul a c) (mul c b))
        (add (mul a c) (mul b c))
        (cong Nat Nat
          (mul c a)
          (mul a c)
          (λx. add x (mul c b))
          (mul_comm c a))
        (cong Nat Nat
          (mul c b)
          (mul b c)
          (λx. add (mul a c) x)
          (mul_comm c b))))

fn mul_assoc_ind (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (mul a (mul b c)) (mul (mul a b) c) =
  match c {
    Zero => tt ;
    Suc c2 =>
      trans Nat
        (mul a (mul b (Suc c2)))
        (add (mul a (mul b c2)) (mul a b))
        (mul (mul a b) (Suc c2))
        (mul_add_distrib_r a (mul b c2) b)
        (cong Nat Nat
          (mul a (mul b c2))
          (mul (mul a b) c2)
          (λx. add x (mul a b))
          (mul_assoc_ind a b c2))
  }

lemma mul_assoc (a : Nat) (b : Nat) (c : Nat)
  : Equal Nat (mul a (mul b c)) (mul (mul a b) c) = mul_assoc_ind a b c
```

## How the document is layered

`add` and `mul` recurse on their second argument and remain the only
value-producing definitions. Each `_ind` helper contains exactly the structural
recursion needed by its headline lemma; the public theorem itself stays
non-recursive and statement-first.

## Using it

The operations remain ordinary values and reduce on concrete naturals.

```ken example
const add_two_three : Nat = add (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))
const mul_two_three : Nat = mul (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))
```

## Trust derivation

The operations, headline lemmas, and recursive helpers are ordinary checked
definitions. Structural recursion is on `Nat`; no trusted declaration or
numeric instance is introduced.
