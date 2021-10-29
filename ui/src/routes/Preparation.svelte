<script lang="ts">
    export let redirect;
    export let data;

    async function getSteps() {
        let response = await fetch(`http://localhost:9000/v1/lists/${data.list_id}/items/${data.item_id}/preparation`, {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        let list = await response.json();
        return list;
    }
</script>

<main>
    <table class="table-auto w-full">
        <thead>
            <tr class="font-semibold tracking-wider text-left bg-gray-100 uppercase border-b border-gray-400">
                <th class="p-3">Name</th>
                <th class="p-3">Start</th>
            </tr>
        </thead>
        <tbody>
            {#await getSteps()}
                <p>Loading</p>
            {:then steps}
                {#each steps as step}
                    <tr class="border">
                        <td class="p-3">{step.name}</td>
                        <td class="p-3">{step.start.days} days before</td>
                    </tr>
                {/each}
            {:catch error}
                <p>Error: {error}</p>
            {/await}
        </tbody>
    </table>
</main>
