# Design System Specification: The Arctic Monolith

## 1. Overview & Creative North Star
The Creative North Star for this design system is **"The Arctic Monolith."** 

In an industry often cluttered by "techno-brutalism" or generic SaaS dashboards, this system seeks to bridge the gap between high-stakes blockchain precision and high-end editorial clarity. We treat the interface not as a grid of boxes, but as a series of sculpted, translucent layers carved from ice and light. 

To achieve this, we move away from "standard" UI patterns. We embrace **intentional asymmetry**, where large Display typography anchors the eye while data-heavy components breathe within expansive white space. The layout should feel like a premium physical ledger: authoritative, permanent, yet effortlessly sophisticated.

---

## 2. Colors & Surface Philosophy
The palette is rooted in a "Nordic Blues" spectrum, transitioning from the deep, midnight depths of the ocean to the piercing clarity of glacial ice.

### The "No-Line" Rule
**Explicit Instruction:** Designers are prohibited from using 1px solid borders for sectioning or containment. Structural boundaries must be defined exclusively through background color shifts or tonal transitions.
- Use `surface` (#f8f9fa) as your base canvas.
- Define internal sections using `surface_container_low` (#f3f4f5) or `surface_container` (#edeeef).
- Contrast is achieved through "Tonal Proximity," not mechanical lines.

### Surface Hierarchy & Nesting
Treat the UI as a physical stack of materials. 
- **The Base:** `surface` (#f8f9fa).
- **The Secondary Tier:** `surface_container_low` for large sidebar or footer areas.
- **The High-Priority Layer:** `surface_container_lowest` (#ffffff) for primary content cards or focal points. This creates a "lifted" effect without artificial shadows.

### The Glass & Gradient Rule
To prevent the UI from feeling "flat" or "cheap," use Glassmorphism for floating elements (modals, dropdowns, navigation bars).
- **Token Usage:** Combine `surface_container_lowest` at 80% opacity with a `backdrop-blur` of 20px.
- **Signature Textures:** Use a subtle linear gradient from `primary` (#030813) to `primary_container` (#1a202c) for primary CTAs to provide a sense of "ink-like" depth.

---

## 3. Typography: The Editorial Voice
This system uses a dual-font approach to balance human approachability with technical rigor.

- **Display & Headlines (Manrope):** Chosen for its geometric, high-tech character. Use `display-lg` (3.5rem) with `-0.02em` tracking for a commanding, editorial presence.
- **Body & Labels (Inter):** The workhorse for the ledger. Use `body-md` (0.875rem) with generous `0.01em` tracking to ensure the blockchain data feels airy and readable.

**Hierarchy Strategy:** 
Large headlines should feel "anchored" to the left, while body text follows a strict vertical rhythm. Use `label-md` in `on_surface_variant` (#45474c) for metadata to ensure it feels secondary but professional.

---

## 4. Elevation & Depth
Depth in this system is a result of light physics, not drop-shadow presets.

### The Layering Principle
Achieve hierarchy by "stacking" tiers. A `surface_container_highest` element placed within a `surface` section creates a natural "well" or "inset" feel. This is the preferred method for data input zones.

### Ambient Shadows
When a component must float (e.g., a critical notification), use an **Ambient Shadow**:
- **Color:** `on_surface` (#191c1d) at 4% opacity.
- **Blur:** 40px to 60px.
- **Y-Offset:** 20px.
This mimics natural light passing through frosted glass rather than a heavy "web" shadow.

### The "Ghost Border" Fallback
If accessibility requirements (WCAG) demand a container edge, use a **Ghost Border**:
- **Token:** `outline_variant` (#c6c6cc) at 15% opacity. 
- **Constraint:** Never use 100% opaque borders; they break the "Arctic" flow.

---

## 5. Components

### Buttons
- **Primary:** Background `primary` (#030813), Text `on_primary` (#ffffff). Shape: `md` (0.75rem). Use `primary_container` for hover states to maintain the navy aesthetic.
- **Secondary:** Background `secondary_fixed` (#d2e4ff), Text `on_secondary_fixed` (#001d37). This provides the "Ice Blue" accent for secondary actions.
- **Tertiary/Ghost:** No background. Text `primary`. Use for low-emphasis utility actions.

### Cards & Lists
- **Rule:** **Strictly forbid divider lines.** 
- Separate list items using vertical spacing (16px - 24px) or a alternating subtle background shift using `surface_container_lowest` and `surface_container_low`.
- **Corner Radius:** Use `lg` (1rem) for external cards to emphasize the "Monolith" feel.

### Input Fields
- **Background:** `surface_container_highest` (#e1e3e4).
- **State:** On focus, transition the background to `surface_container_lowest` (#ffffff) and apply the "Ghost Border." This gives the user the feeling of "lighting up" a field.

### Signature Component: The "Ledger Glass" Data Table
To bridge marketing and blockchain, data tables should use `surface_container_lowest` with a slight `secondary_container` (#66affe) glow on the active row to guide the eye without adding visual weight.

---

## 6. Do’s and Don’ts

### Do:
- **Use "White Space as a Border":** If two elements feel too close, increase the padding, don't add a line.
- **Embrace Asymmetry:** Place a `headline-lg` off-center to create a modern, editorial vibe.
- **Use Tonal Layers:** Nest a `surface_container_highest` card inside a `surface_container_low` wrapper.

### Don't:
- **Don't use #000000:** It is too harsh for this system. Use `primary` (#030813) for the deepest blacks.
- **Don't use 1px borders:** This is the most critical rule to maintain the "Nordic" aesthetic.
- **Don't use high-contrast shadows:** Avoid the "Material Design 2" look. Keep shadows large, soft, and nearly invisible.
- **Don't crowd data:** Blockchain applications are complex; use the typography scale to prioritize the most important hash or value, and let the rest recede.