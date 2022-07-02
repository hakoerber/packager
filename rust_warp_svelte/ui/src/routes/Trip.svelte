<script lang="ts">
    export let redirect;
    export let data;

    export const url = "/trips/"

    async function getTrip() {
        let response = await fetch(`http://localhost:9000/v1/trips/${data.id}`, {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        let trip = await response.json();
        return trip;
    }

    async function getTripItems() {
        let response = await fetch(`http://localhost:9000/v1/trips/${data.id}/items`, {
            method: "GET",
            headers: {
                "Accept": "application/json"
            },
            cache: "no-store",
        });
        let items = await response.json();
        return items;
    }
</script>

<main>
    <div class="container mx-auto mt-12">
        {#await getTrip()}
            <p>Loading</p>
        {:then trip}
            <h2 class="text-3xl mt-12 mb-20 font-semibold text-center mb-5 mt-3">{trip.name}</h2>

            <table>
                <tr>
                    <td>Date</td>
                    <td>{trip.date}</td>
                </tr>
                <tr>
                    <td>Duration</td>
                    <td>{trip.parameters.days} Days</td>
                </tr>
                <tr>
                    <td>Status</td>
                    <td>{trip.state}</td>
                </tr>
            </table>

            <table class="table-auto w-full">
                <thead>
                    <tr class="font-semibold tracking-wider text-left bg-gray-100 uppercase border-b border-gray-400">
                        <th class="p-3">Name</th>
                        <th class="p-3">Status</th>
                    </tr>
                </thead>
                <tbody>
                    {#await getTripItems()}
                        <p>Loading</p>
                    {:then items}
                        {#each items as item}
                            <tr class="border">
                                <td class="p-3">{item.packageItem.name}</td>
                                <td class="p-3">{item.status}</td>
                            </tr>
                        {/each}
                    {:catch error}
                        {error}
                    {/await}
                </tbody>
            </table>
        {/await}
    </div>
</main>
