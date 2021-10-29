<script lang="ts">
    export let redirect;
    export let data;

    export const url = "/trips/"

    async function getTrips() {
        let response = await fetch("http://localhost:9000/v1/trips", {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        let trips = await response.json();
        return trips;
    }
</script>

<main>
    <div class="container mx-auto mt-12">
        <h2 class="text-3xl mt-12 mb-20 font-semibold text-center mb-5 mt-3">Trips</h2>
        <table class="table-auto w-full">
            <thead>
                <tr class="font-semibold tracking-wider text-left bg-gray-100 uppercase border-b border-gray-400">
                    <th class="p-3">Name</th>
                    <th class="p-3">Date</th>
                    <th class="p-3">Days</th>
                    <th class="p-3">State</th>
                    <th class="p-3">Package Lists</th>
                </tr>
            </thead>
            <tbody>
                {#await getTrips()}
                    <p>Loading</p>
                {:then trips}
                    {#each trips as trip}
                        <tr class="border" on:click={e => redirect(url + trip.id)}>
                            <td class="p-3">{trip.name}</td>
                            <td class="p-3">{trip.date}</td>
                            <td class="p-3">{trip.parameters.days}</td>
                            {#if trip.state == "active"}
                                <td class="p-3 bg-green-100">{trip.state}</td>
                            {:else if trip.state == "planned"}
                                <td class="p-3 bg-blue-100">{trip.state}</td>
                            {:else}
                                <td class="p-3">{trip.state}</td>
                            {/if}
                            <td class="p-3">
                                <ul>
                                {#each trip.packageLists as list}
                                    <li><button on:click={() => redirect(`/lists/${list.id}`)}>{list.name}</button></li>
                                {/each}
                                </ul>
                            </td>
                        </tr>
                    {/each}
                {:catch error}
                    <p>Error: {error}</p>
                {/await}
            </tbody>
        </table>
    </div>
</main>
