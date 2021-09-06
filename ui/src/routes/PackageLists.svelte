<script lang="ts">
    import PackageList from "../components/PackageList.svelte"

    async function getUsers() {
        let response = await fetch("http://localhost:9000/v1/lists", {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        let users = await response.json();
        return users;
    }
</script>

<main>
    <div>
        {#await getUsers()}
            <p>Loading</p>
        {:then lists}
            <div class="m-2 grid grid-cols-3 gap-5 items-start grid-flow-row">
                {#each lists as list}
                    <div class="p-3 border rounded-lg border-gray-300 shadow hover:shadow-xl bg-gray-100 bg-opacity-30 hover:bg-opacity-100">
                        <PackageList id={list.id} name={list.name} items={list.items}/>
                    </div>
                {/each}
            </div>
        {:catch error}
            <p>Something went wrong</p>
        {/await}
    </div>
</main>
