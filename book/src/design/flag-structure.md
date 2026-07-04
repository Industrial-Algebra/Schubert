# Flag Structure in Access Control

> Addresses Proserpina critique finding #9: "Schubert varieties are defined
> relative to a fixed complete flag, but the document never explains what
> 'flag' means in an access control context."

## What Is a Flag?

A **complete flag** in an n-dimensional vector space V is a nested sequence
of subspaces:

```
{0} = V₀ ⊂ V₁ ⊂ V₂ ⊂ ... ⊂ Vₙ = V
```

where each Vᵢ has dimension i.

Schubert varieties are defined *relative to* a fixed flag. The flag
determines the geometry of each Schubert condition — different flags give
different Schubert varieties even for the same partition.

## The Flag in Access Control

In Schubert's access control model, the flag represents the **hierarchy of
security clearance levels** or **trust zones**:

```
V₀ = {0}              — No access (empty subspace)
V₁ = 1-dim            — Public data (read-only)
V₂ = 2-dim            — Internal data (read + limited write)
V₃ = 3-dim            — Confidential data (read + write + audit)
V₄ = V (full space)   — Secret data (full admin)
```

Each level in the flag is a progressively larger subspace — more capabilities,
more access.

## How the Flag Determines Schubert Conditions

A Schubert condition σ_λ is defined by how the principal's state subspace
intersects the flag subspaces. The partition λ encodes the intersection
pattern:

| Partition | Intersection with Flag | Access Control Meaning |
|---|---|---|
| [1] | State meets V₁ but is generic in V₂ | Read public data |
| [2] | State meets V₁ and satisfies a codim-2 condition in V₂ | Write internal data |
| [1,1] | State meets V₁ in a codim-2 way (different from [2]) | Audit internal data |
| [2,2] | State is a point — meets all flag levels maximally | Full admin (point class) |

**Why σ₂ and σ₁₁ differ:** Both have codimension 2, but they interact with
the flag differently:

- **σ₂ [2]:** The state meets V₁ generically, then satisfies a codimension-2
  condition in V₂. This means: "can read public data AND has write access to
  internal data."

- **σ₁₁ [1,1]:** The state satisfies a codimension-1 condition in V₁ AND a
  codimension-1 condition in V₂/V₁. This means: "has restricted read access
  to public data AND restricted access to internal data."

These are geometrically different constraints even though they have the same
codimension. That's why σ₂·σ₁₁ can be zero — the constraints pull the state
in incompatible directions relative to the flag.

## Choosing a Flag for Your Application

The flag is application-specific. Different domains have different hierarchies:

### Web Application (Gr(2,4))

```
V₀ = No access
V₁ = Anonymous (public pages)
V₂ = Authenticated (user dashboard)
V₃ = Staff (admin panel)
V₄ = Root (system config)
```

### Distributed Game (Gr(2,4))

```
V₀ = Disconnected
V₁ = Spectator (view-only)
V₂ = Player (move + interact)
V₃ = Moderator (kick + ban)
V₄ = Admin (server config)
```

### Multi-Agent Coding Harness (Gr(2,4))

```
V₀ = No access
V₁ = Reader (clone + view)
V₂ = Contributor (branch + push)
V₃ = Reviewer (approve + merge)
V₄ = Maintainer (deploy + release)
```

## Flag-Agnostic Properties

Some Schubert properties are flag-independent:

- The total codimension (sum of partition entries) is the same for any flag.
- The Grassmannian dimension k(n-k) is flag-independent.
- Commutativity of intersection (σ_a · σ_b = σ_b · σ_a) holds for any flag.

Flag-dependent properties:

- The specific intersection number can depend on the flag for non-standard
  conditions. For the standard Schubert classes used in Schubert (σ₁, σ₂,
  σ₁₁, etc.), the intersection numbers are flag-independent classical
  results.

## Practical Note

For most applications, you don't need to think about the flag explicitly.
The `AccessController::new(k, n)` constructor sets up the standard flag
automatically. The flag becomes relevant only when:

1. You need a custom clearance hierarchy
2. You're debugging why two same-codimension conditions have different
   intersection behavior
3. You're writing a formal proof or paper about the system

For the standard use case (RBAC, multi-tenant, game sync), the default
flag works correctly.
