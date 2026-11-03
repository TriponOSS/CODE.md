<script lang="ts">
  import type { SkillInfo } from "$lib/types";
  import Icon from "./Icon.svelte";

  let { skills, onOpen }: { skills: SkillInfo[]; onOpen: (path: string) => void } = $props();

  const invalid = $derived(skills.filter((skill) => !skill.valid).length);
</script>

<div class="flex items-center gap-2 px-3 pt-1 pb-2 text-xs tabular-nums text-faint">
  <span>{skills.length} {skills.length === 1 ? "skill" : "skills"}</span>
  {#if invalid > 0}
    <span class="text-danger">{invalid} need{invalid === 1 ? "s" : ""} a fix</span>
  {:else if skills.length > 0}
    <span class="text-ok">all valid</span>
  {/if}
</div>

{#if skills.length === 0}
  <div class="px-4.5 py-7">
    <p class="m-0 mb-1.5 text-[13px] font-medium text-ink">No skills found</p>
    <p class="m-0 text-xs leading-relaxed text-faint">
      Add a <code class="font-mono text-accent">SKILL.md</code> with YAML frontmatter anywhere in this folder, then rescan.
    </p>
  </div>
{:else}
  <div class="py-1 pb-4">
    {#each skills as skill (skill.path)}
      <button
        type="button"
        class="mx-2 my-0.5 flex w-[calc(100%-16px)] flex-col gap-1 rounded-lg p-3 text-left transition-colors duration-150 hover:bg-hover"
        title={skill.path}
        onclick={() => onOpen(skill.path)}
      >
        <span class="flex items-center justify-between gap-2">
          <span class="truncate text-[13px] font-medium text-ink">{skill.name}</span>
          {#if skill.valid}
            <span class="flex shrink-0 items-center gap-0.5 text-[11px] font-medium text-ok">
              <Icon name="check" size={13} /> ready
            </span>
          {:else}
            <span class="flex shrink-0 items-center gap-0.5 text-[11px] font-medium text-danger">
              <Icon name="alert" size={13} /> needs fix
            </span>
          {/if}
        </span>

        {#if skill.description}
          <span class="line-clamp-2 text-xs leading-normal text-muted">{skill.description}</span>
        {/if}

        {#if skill.triggers.length > 0}
          <span class="truncate text-[11.5px] leading-snug text-faint">{skill.triggers.join(" · ")}</span>
        {/if}

        {#if skill.issues.length > 0}
          <span class="mt-px text-[11.5px] leading-snug text-danger">
            {#each skill.issues as issue, index (issue)}
              {#if index > 0}<span class="text-line-strong"> · </span>{/if}{issue}
            {/each}
          </span>
        {/if}

        <span class="truncate font-mono text-[11px] text-faint">{skill.path}</span>
      </button>
    {/each}
  </div>
{/if}
