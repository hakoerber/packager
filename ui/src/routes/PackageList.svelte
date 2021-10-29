<script lang="ts">
    import { onMount } from 'svelte';

    import PackageListTable from "../components/PackageListTable.svelte"

    export let redirect;
    export let data;

    const resetActiveElement = () => {
        activeElement = {
            name: "",
            count: 1,
            preparationsteps: [],
        };
    };

    let activeElement;
    resetActiveElement();

    export const url = `/lists/${data.id}`

    let sidebarActive = true;

    const toggleSidebar = () => {
        sidebarActive = !sidebarActive;
    }

    const apply = async () => {
        const response = await fetch(`http://localhost:9000/v1/lists/${data.id}/items`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "Accept": "application/json",
            },
            body: JSON.stringify({
                name: activeElement.name,
                count: activeElement.count,
            }),
            cache: "no-store",
        });

        const d = await response.json();

        console.log(d);
        items = [...items, d];
        console.log(items[0]);
        console.log(d);

        resetActiveElement();
        sidebarActive = false;
    }

    const cancel = () => {
        resetActiveElement();
        sidebarActive = false;
    }

    async function getItems(id) {
        let response = await fetch(`http://localhost:9000/v1/lists/${id}/items`, {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        items = await response.json();
    }

    async function getList() {
        const response = await fetch(`http://localhost:9000/v1/lists/${data.id}`, {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        })

        list = await response.json();
    }

    let list = {name: ""};
    let items = [];

    onMount(async () => {
        await getList();
        await getItems(data.id);
	});
</script>

<main>
    <div>
        <h2 class="text-3xl mt-12 mb-20 font-semibold text-center mb-5 mt-3">{list.name}</h2>
        <div class="container mx-auto grid grid-cols-12 gap-1 items-start justify-items-stretch">
            <div class="col-start-1 col-end-9">
                <PackageListTable items={items} id={list.id} {redirect}/>
                <button class="p-3 w-full mt-3 border border-gray-200 bg-indigo-300" on:click={toggleSidebar}>Add new item</button>
            </div>
            <div class="col-start-9 col-end-10"/>
            <div class="col-start-10 col-end-13">
                {#if sidebarActive}
                    <div>
                        <label for="name">Name</label>
                        <input
                            class="w-full"
                            type="text"
                            id="name"
                            name="name"
                            bind:value={activeElement.name}
                        />
                    </div>
                    <div>
                        <label for="count">Count</label>
                        <input
                            class="w-full"
                            type="number"
                            id="count"
                            name="count"
                            bind:value={activeElement.count}
                        />
                    </div>
                    <div>
                        {#each activeElement.preparationsteps as step}
                            {step}
                        {/each}
                    </div>
                    <div class="flex flex-row mt-6 justify-between w-full">
                        <button type="submit" class="p-3 border border-gray-200 bg-green-300" on:click={() => apply()}>Apply</button>
                        <button class="p-3 border border-gray-200 bg-red-300" on:click={() => cancel()}>Cancel</button>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</main>
