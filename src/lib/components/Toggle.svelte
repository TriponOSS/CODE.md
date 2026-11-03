<script lang="ts">
  let {
    checked,
    label,
    onChange,
  }: {
    checked: boolean;
    label: string;
    onChange: (checked: boolean) => void;
  } = $props();
</script>

<button
  class="relative h-5 w-9 shrink-0 rounded-full border transition-colors duration-150 {checked ? 'border-ink bg-ink' : 'border-line-strong bg-active'}"
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  onclick={() => onChange(!checked)}
>
  <span class="absolute top-0.5 size-3.5 rounded-full transition-[left,background-color] duration-150 {checked ? 'left-[18px] bg-app' : 'left-0.5 bg-faint'}"></span>
</button>
