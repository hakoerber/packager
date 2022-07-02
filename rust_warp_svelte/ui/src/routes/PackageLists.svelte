<script lang="ts">
    import PackageList from "../components/PackageList.svelte"

    export let redirect;
    export let data;

    export const url = "/lists/"

    async function getLists() {
        let response = await fetch("http://localhost:9000/v1/lists", {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        let lists = await response.json();
        return lists;
    }
</script>

<main>
    <div class="container mx-auto mt-12">
        <table class="table-auto w-full">
            <thead>
                <tr class="font-semibold tracking-wider text-left bg-gray-100 uppercase border-b border-gray-400">
                    <th class="p-3">Name</th>
                    <th class="p-3"># Items</th>
                </tr>
            </thead>
            <tbody>
                {#await getLists()}
                    <p>Loading</p>
                {:then lists}
                    {#each lists as list}
                        <tr class="border" on:click={e => redirect(url + list.id)}>
                            <td class="p-3">{list.name}</td>
                            <td class="p-3">{list.items.length}</td>
                        </tr>
                    {/each}
                {:catch error}
                    <p>Error: {error}</p>
                {/await}
            </tbody>
        </table>
    </div>
</main>
