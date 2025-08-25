// Given voltage files, add their entire contents and report the sum.

#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include "fitsio.h"
#include "mwalib.h"

#define ERROR_MESSAGE_LEN 1024

typedef struct {
    VoltageContext *context;
    char *error_message;
    long num_bytes_per_cc_per_timestep;
    int timestep_index;
    int coarse_chan_index;
    long local_sum;
} ThreadArgs;

void *process_coarse_channel(void *arg) {
    ThreadArgs *args = (ThreadArgs *)arg;

    signed char *buffer = calloc(args->num_bytes_per_cc_per_timestep, sizeof(signed char));
    if (!buffer) {
        perror("calloc");
        pthread_exit(NULL);
    }

    int ret = mwalib_voltage_context_read_file(args->context,
                                               args->timestep_index,
                                               args->coarse_chan_index,
                                               buffer,
                                               args->num_bytes_per_cc_per_timestep,
                                               args->error_message,
                                               ERROR_MESSAGE_LEN);

    if (ret == MWALIB_SUCCESS) {
        long sum = 0;
        for (long i = 0; i < args->num_bytes_per_cc_per_timestep; i++) {
            sum += buffer[i];
        }
        args->local_sum = sum;
        printf("Timestep: %d, Coarse channel: %d, Sum: %ld\n",
               args->timestep_index, args->coarse_chan_index, sum);
    }
    else if (ret == MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN) {
        args->local_sum = 0;
    }
    else {
        printf("Error: %s\n", args->error_message);
        free(buffer);
        pthread_exit((void*)-1);
    }

    free(buffer);
    pthread_exit(NULL);
}

void do_sum_parallel_pthreads(VoltageContext *context,
                              long num_bytes_per_cc_per_timestep,
                              int num_timesteps,
                              int num_coarse_chans)
{
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));
    long total_sum = 0;

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++) {
        pthread_t threads[num_coarse_chans];
        ThreadArgs args[num_coarse_chans];

        // Create threads — one per coarse channel
        for (int coarse_chan_index = 0; coarse_chan_index < num_coarse_chans; coarse_chan_index++) {
            args[coarse_chan_index].context = context;
            args[coarse_chan_index].error_message = error_message;
            args[coarse_chan_index].num_bytes_per_cc_per_timestep = num_bytes_per_cc_per_timestep;
            args[coarse_chan_index].timestep_index = timestep_index;
            args[coarse_chan_index].coarse_chan_index = coarse_chan_index;
            args[coarse_chan_index].local_sum = 0;

            if (pthread_create(&threads[coarse_chan_index], NULL,
                               process_coarse_channel, &args[coarse_chan_index]) != 0) {
                perror("pthread_create");
                exit(EXIT_FAILURE);
            }
        }

        // Join threads and accumulate timestep sum
        long timestep_sum = 0;
        for (int coarse_chan_index = 0; coarse_chan_index < num_coarse_chans; coarse_chan_index++) {
            pthread_join(threads[coarse_chan_index], NULL);
            timestep_sum += args[coarse_chan_index].local_sum;
        }

        total_sum += timestep_sum;
    }

    printf("Total sum: %ld\n", total_sum);
    free(error_message);
}

int main(int argc, char *argv[])
{
    // Assume that the first file provided is the metafits file, and all others
    // are gpubox files. Therefore, we need at least two files provided to main,
    // such that there's at least one gpubox file.
    if (argc < 3)
    {
        printf("At least two files are needed.\n");
        return EXIT_FAILURE;
    }

    const char **volt_files = malloc(sizeof(char *) * (argc - 2));
    for (int i = 0; i < argc - 2; i++)
    {
        volt_files[i] = argv[i + 2];
    }

    // Allocate buffer for any error messages
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));

    VoltageContext *voltage_context = NULL;

    if (mwalib_voltage_context_new(argv[1], volt_files, argc - 2, &voltage_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error creating voltage context: %s\n", error_message);
        exit(-1);
    }

    // Print summary of voltage context
    if (mwalib_voltage_context_display(voltage_context, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error displaying voltage context summary: %s\n", error_message);
        exit(-1);
    }

    VoltageMetadata *voltage_metadata = NULL;

    if (mwalib_voltage_metadata_get(voltage_context, &voltage_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting voltage metadata: %s\n", error_message);
        exit(-1);
    }

    MetafitsMetadata *metafits_metadata = NULL;

    if (mwalib_metafits_metadata_get(NULL, NULL, voltage_context, &metafits_metadata, error_message, ERROR_MESSAGE_LEN) != EXIT_SUCCESS)
    {
        printf("Error getting metafits metadata: %s\n", error_message);
        exit(-1);
    }

    int num_timesteps = voltage_metadata->num_timesteps;
    int num_coarse_chans = voltage_metadata->num_coarse_chans;
    long num_bytes_per_cc_per_ts = voltage_metadata->expected_voltage_data_file_size_bytes;
    
    do_sum_parallel_pthreads(voltage_context, num_bytes_per_cc_per_ts, num_timesteps, num_coarse_chans);

    mwalib_metafits_metadata_free(metafits_metadata);
    mwalib_voltage_metadata_free(voltage_metadata);
    mwalib_voltage_context_free(voltage_context);
    free(volt_files);
    free(error_message);

    return EXIT_SUCCESS;
}