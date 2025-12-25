<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import { Button } from "$lib/components/ui/button";
  import { ListTodo, Container, Settings } from "lucide-svelte";
  import { DockersPage } from "$lib/features/dockers";

  let activeTab = $state("tasks");
</script>

<div class="flex h-screen bg-background text-foreground">
  <!-- Sidebar -->
  <nav class="w-56 border-r border-border bg-card p-4">
    <h1 class="mb-6 text-xl font-bold">rustation</h1>

    <Tabs.Root bind:value={activeTab} orientation="vertical" class="w-full">
      <Tabs.List class="flex flex-col gap-1 bg-transparent">
        <Tabs.Trigger
          value="tasks"
          class="justify-start gap-2 data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
        >
          <ListTodo class="h-4 w-4" />
          Tasks
        </Tabs.Trigger>
        <Tabs.Trigger
          value="dockers"
          class="justify-start gap-2 data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
        >
          <Container class="h-4 w-4" />
          Dockers
        </Tabs.Trigger>
        <Tabs.Trigger
          value="settings"
          class="justify-start gap-2 data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
        >
          <Settings class="h-4 w-4" />
          Settings
        </Tabs.Trigger>
      </Tabs.List>
    </Tabs.Root>
  </nav>

  <!-- Main Content -->
  <main class="flex-1 overflow-auto p-6">
    {#if activeTab === "tasks"}
      <div>
        <h2 class="text-2xl font-semibold">Tasks</h2>
        <p class="mt-2 text-muted-foreground">Run justfile commands</p>
        <div class="mt-6">
          <Button>Refresh</Button>
        </div>
      </div>
    {:else if activeTab === "dockers"}
      <DockersPage />
    {:else if activeTab === "settings"}
      <div>
        <h2 class="text-2xl font-semibold">Settings</h2>
        <p class="mt-2 text-muted-foreground">Configuration</p>
      </div>
    {/if}
  </main>
</div>
