<script lang="ts">
    import PackageItem from "./PackageItem.svelte"
    import { fade, fly, draw } from 'svelte/transition';

    export let data;

    let active = false;
    let shown_items = [];
    let ellipsed = false;

    const enter = () => {
        active = true
    }
    const leave = () => {
        active = false
    }

    $: if (data.items.length <= 4) {
        shown_items = data.items
        ellipsed = false
    } else {
        shown_items = data.items.slice(0, !active && 3 || data.items.length)
        ellipsed = true
    }
</script>

<main on:mouseenter={enter} on:mouseleave={leave}>
    <h2 class="text-lg font-semibold text-center mb-5 mt-3">{data.name}</h2>
    <ul class="list-disc ml-5">
    {#each shown_items as item}
        <li in:fade|local={{duration: 200}} out:fade|local={{duration: 100, delay: 100}}><PackageItem data={item} /></li>
    {/each}
    {#if !active && ellipsed}
        ...
    {/if}
    </ul>
</main>

<style>
</style>
