<svelte:head>
    <link href="https://unpkg.com/tailwindcss@^2/dist/tailwind.min.css" rel="stylesheet">
</svelte:head>

<script lang="ts">
    import PackageList from "./PackageList.svelte"
	export let name: string;

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

    const promise = getUsers()
</script>

<main>
    <div>
        {#await promise}
            <p>Loading</p>
        {:then lists}
            <div class="m-2 grid grid-cols-3 gap-5 items-start grid-flow-row">
                {#each lists as list}
                    <div class="p-3 border rounded-lg border-gray-300 shadow hover:shadow-xl bg-gradient-to-br bg-blue-300 bg-opacity-50 hover:bg-opacity-100">
                        <PackageList data={list} />
                    </div>
                {/each}
            </div>
        {:catch error}
            <p>Something went wrong</p>
        {/await}
    </div>
</main>
