<script lang="ts">
    export let timeString: string;
    export let dateString: string;
</script>

<div class="panel">
    <div class="panel-box">
        <div class="flex flex-col items-center">
            <span class="title">{timeString}</span>
            <span class="subtitle">{dateString}</span>
        </div>
    </div>
</div>