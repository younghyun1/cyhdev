import { pageStyles } from "../styles/pageStyles";

export default function NotFound() {
  return (
    <main class={pageStyles.page}>
      <div
        class={`${pageStyles.pageInnerNarrow} flex items-center justify-center min-h-[70vh]`}
      >
        <div class={`${pageStyles.cardPadded} w-full`}>
          <div class="flex items-start gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300 ring-1 ring-amber-200/60 dark:ring-amber-500/30">
              <span class="text-xl" aria-hidden="true">
                🚧
              </span>
            </div>

            <div class="min-w-0">
              <h1 class={`${pageStyles.titleSm} mb-1`}>Under Construction</h1>
              <p class={pageStyles.muted}>I&apos;m working on it!</p>
            </div>
          </div>

          <div class="mt-6 grid gap-4 sm:grid-cols-3">
            <div class={`${pageStyles.card} p-4`}>
              <p class="text-sm font-semibold">Status</p>
              <p class={`mt-1 ${pageStyles.muted}`}>In progress</p>
            </div>
            <div class={`${pageStyles.card} p-4`}>
              <p class="text-sm font-semibold">ETA</p>
              <p class={`mt-1 ${pageStyles.muted}`}>Soon™</p>
            </div>
            <div class={`${pageStyles.card} p-4`}>
              <p class="text-sm font-semibold">Meanwhile</p>
              <p class={`mt-1 ${pageStyles.muted}`}>Drink water.</p>
            </div>
          </div>

          <div class="mt-6 flex flex-wrap gap-3">
            <a href="/" class={pageStyles.buttonPrimary}>
              Go to homepage
            </a>
            <button
              type="button"
              onClick={() => history.back()}
              class={pageStyles.buttonSecondary}
            >
              Go back
            </button>
          </div>
        </div>
      </div>
    </main>
  );
}
