<script lang="ts">
    import PackageItem from "./PackageItem.svelte"
    import { fade, fly, draw } from 'svelte/transition';

    export let name;
    export let items;
    export let id;

    let active = false;
    let shown_items = [];
    let ellipsed = false;

    const enter = () => {
        active = true
    }
    const leave = () => {
        active = false
    }

    $: if (items.length <= 4) {
        shown_items = items
        ellipsed = false
    } else {
        shown_items = items.slice(0, !active && 3 || items.length)
        ellipsed = true
    }

    function onClick(e) {
        window.location = `/lists/${e}`;
    }
</script>

<main on:mouseenter={enter} on:mouseleave={leave} on:click={onClick(id)}>
    <h2 class="text-lg font-semibold text-center mb-5 mt-3">{name}</h2>
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
