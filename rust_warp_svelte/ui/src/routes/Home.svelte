<script lang="ts">
    export let redirect;

    function navigateToPackageLists() {
        redirect("/lists/");
    }
</script>

<main>
    <div>
        Welcome to Packager, your helper for your next trip!
    </div>
    <button on:click={navigateToPackageLists}>Lists</button>
    <button on:click={() => redirect("/trips/")}>Trips</button>
</main>
