<script lang="ts">
    import PackageListTable from "../components/PackageListTable.svelte"

    export let redirect;
    export let data;

    export const url = `/lists/${data.id}`

    async function getList() {
        let response = await fetch(`http://localhost:9000/v1/lists/${data.id}`, {
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
    <div>
        {#await getList()}
            <p>Loading</p>
        {:then list}
            <div class="container mx-auto">
                <h2 class="text-3xl mt-12 mb-20 font-semibold text-center mb-5 mt-3">{list.name}</h2>
                <PackageListTable list={list} />
            </div>
        {:catch error}
            <p>Error: {error}</p>
        {/await}
    </div>
</main>
