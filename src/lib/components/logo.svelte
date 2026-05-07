<script>
  let { active = true, class: className = "" } = $props();
</script>

<svg
  class={className}
  width="28"
  height="28"
  viewBox="0 0 1080 1080"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
>
  <!-- Orange base -->
  <path
    d="M547.866 501.658C564.015 582.381 477.782 889.125 450.975 953.703H580.163C644.757 776.114 564.015 566.236 547.866 501.658Z"
    fill={active ? "#FB6D49" : "currentColor"}
    stroke={active ? "#FB6D49" : "currentColor"}
    stroke-width="64.5859"
    stroke-linecap="round"
    stroke-linejoin="round"
  />

  <!-- Left curve -->
  <path
    d="M533.5 568.5C583 531 546.103 359.464 487.74 302.287C429.377 245.111 340.009 241.837 288.086 294.811L326.483 332.427L364.879 370.043"
    fill={active ? "#AFF33E" : "currentColor"}
    stroke={active ? "#AFF33E" : "currentColor"}
    stroke-width="53.7516"
    stroke-linecap="round"
    stroke-linejoin="round"
  />

  <!-- Right curve -->
  <path
    d="M554.748 559.534C502.826 506.561 516.187 300.6 574.549 243.424C632.912 186.247 701.115 195.692 791.666 227.266L753.269 264.882L678.145 314.571"
    fill={active ? "#AFF33E" : "currentColor"}
    stroke={active ? "#AFF33E" : "currentColor"}
    stroke-width="53.7516"
    stroke-linecap="round"
    stroke-linejoin="round"
  />

  <!-- Bottom left arc -->
  <path
    d="M554.93 501.632C554.93 427.464 488.808 367.27 407.095 367.27C325.382 367.27 259.26 427.464 259.26 501.632H313.018H366.776H474.293"
    fill={active ? "#AFF33E" : "currentColor"}
    stroke={active ? "#AFF33E" : "currentColor"}
    stroke-width="53.7516"
    stroke-linecap="round"
    stroke-linejoin="round"
  />

  <!-- Bottom right arc -->
  <path
    d="M554.931 478.519C582.149 458.401 615.159 447.651 649.008 447.884C730.72 447.884 796.843 508.078 796.843 582.246V585L690 578.5L662 576L554.931 562"
    fill={active ? "#AFF33E" : "currentColor"}
    stroke={active ? "#AFF33E" : "currentColor"}
    stroke-width="53.7516"
    stroke-linecap="round"
    stroke-linejoin="round"
  />

  <!-- Top arc -->
  <path
    d="M499.547 373.69C520.226 346.899 549.039 327.537 581.662 318.509C660.261 296.174 740.326 336.001 760.608 407.343L683.043 429.384L657.621 436.608L631.333 444.078L553.768 466.119"
    fill={active ? "#AFF33E" : "currentColor"}
    stroke={active ? "#AFF33E" : "currentColor"}
    stroke-width="53.7516"
    stroke-linecap="round"
    stroke-linejoin="round"
  />

  <!-- Bottom shape -->
  <path
    d="M363.822 547.583C306.032 605.359 302 694.575 354.415 747.245L468.382 633.037L487.198 614.227L506.282 595.147L563.266 538.178C510.851 485.508 421.612 489.807 363.822 547.583Z"
    fill={active ? "#AFF33E" : "currentColor"}
    stroke={active ? "#AFF33E" : "currentColor"}
    stroke-width="53.7516"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>