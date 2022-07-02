<script lang="ts">
    export let redirect;
</script>

<main>
    <div>
        404 -- Not Found
    </div>
    <button on:click={() => redirect("/")}>Back to home</button>
</main>
