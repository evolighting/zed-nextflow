#!/usr/bin/env nextflow

nextflow.enable.dsl = 2

params.greeting = 'Hello'

process GREET {
    tag "$name"

    input:
    val name

    output:
    stdout

    script:
    """
    echo '${params.greeting}, ${name}!'
    """
}

workflow {
    names = Channel.of('Zed', 'Nextflow')
    GREET(names)
    GREET.out.view()
}
