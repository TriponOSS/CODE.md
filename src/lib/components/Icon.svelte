<script lang="ts">
  import { HugeiconsIcon } from "@hugeicons/svelte";
  import {
    Alert02Icon,
    ArrowRight01Icon,
    Cancel01Icon,
    CheckmarkCircle02Icon,
    ClockIcon,
    ComputerIcon,
    Doc01Icon,
    EyeIcon,
    EyeOffIcon,
    FloppyDiskIcon,
    Folder01Icon,
    HeadingIcon,
    LibraryIcon,
    ListSettingIcon,
    MagicWand01Icon,
    Maximize01Icon,
    MinusIcon,
    Moon02Icon,
    Settings01Icon,
    Sun01Icon,
    Note01Icon,
    PanelRightIcon,
    RefreshIcon,
    Search01Icon,
    TextIcon,
  } from "@hugeicons/core-free-icons";

  const icons = {
    folder: Folder01Icon,
    file: Doc01Icon,
    markdown: Note01Icon,
    skill: MagicWand01Icon,
    search: Search01Icon,
    save: FloppyDiskIcon,
    eye: EyeIcon,
    "eye-off": EyeOffIcon,
    close: Cancel01Icon,
    chevron: ArrowRight01Icon,
    alert: Alert02Icon,
    check: CheckmarkCircle02Icon,
    refresh: RefreshIcon,
    library: LibraryIcon,
    panel: PanelRightIcon,
    outline: HeadingIcon,
    list: ListSettingIcon,
    clock: ClockIcon,
    text: TextIcon,
    minus: MinusIcon,
    maximize: Maximize01Icon,
    settings: Settings01Icon,
    moon: Moon02Icon,
    sun: Sun01Icon,
    computer: ComputerIcon,
  } as const;

  type IconName = keyof typeof icons;

  let {
    name,
    size = 16,
    strokeWidth = 1.5,
  }: { name: IconName; size?: number; strokeWidth?: number } = $props();
</script>

<HugeiconsIcon icon={icons[name]} size={size} strokeWidth={strokeWidth} color="currentColor" />
