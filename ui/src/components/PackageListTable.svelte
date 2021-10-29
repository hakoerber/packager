<script lang="ts">
    export let id;
    export let items;
    export let redirect;

    $: has_counts = items.some(l => l.count > 1);
    $: has_preparations = items.some(l => l.preparation.steps.length > 0);
    //has_sizes = items.some(l => l.size.type != "None");

    let has_sizes = false;
    let has_preparations = false;
    let has_counts = false;

    const navigateToPreparation = (item_id) => {
        redirect(`/lists/${id}/items/${item_id}/preparation`);
    }
</script>

<div>
    <table class="table-auto w-full">
        <thead>
            <tr class="font-semibold tracking-wider text-left bg-gray-100 uppercase border-b border-gray-400">
                <th class="p-3">Name</th>
                {#if has_sizes }
                    <th class="p-3">Size</th>
                {/if}
                {#if has_counts}
                    <th class="p-3">Count</th>
                {/if}
                {#if has_preparations}
                    <th class="p-3">Preparation Steps</th>
                {/if}
            </tr>
        </thead>
        <tbody>
            {#each items as item}
            <tr class="border">
                <td class="p-3">{item.name}</td>
                {#if has_sizes }
                    <td class="p-3">
                        {#if item.size.type == "None"}
                        {:else if item.size.type == "Gram"}
                            {#if item.size.value == 1}
                                {item.size.value} Gram
                            {:else}
                                {item.size.value} Grams
                            {/if}
                        {:else if item.size.type == "Pack"}
                            {#if item.size.value == 1}
                                {item.size.value} Pack
                            {:else}
                                {item.size.value} Packs
                            {/if}
                        {:else}
                            {item.size.value} {item.size.type}
                        {/if}
                    </td>
                {/if}
                {#if has_counts}
                    <td class="p-3">
                        {#if item.count > 1}
                            {item.count}
                        {/if}
                    </td>
                {/if}
                {#if has_preparations}
                    <td class="p-3">
                        {#if item.preparation.steps.length > 0}
                            {item.preparation.steps.length} steps
                            <button on:click={() => navigateToPreparation(item.id)}>Show preparation steps</button>
                        {/if}
                    </td>
                {/if}
            </tr>
            {/each}
        </tbody>
    </table>
</div>
