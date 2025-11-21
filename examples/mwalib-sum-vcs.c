// Given voltage files, add their entire contents and report the sum.

#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>
#include "fitsio.h"
#include "mwalib.h"

#define ERROR_MESSAGE_LEN 1024

typedef struct
{
    VoltageContext *context;
    char *error_message;
    long num_bytes_per_cc_per_timestep;
    int timestep_index;
    int coarse_chan_index;
    long local_sum;
    int version;
} ThreadArgs_read_file;

typedef struct
{
    VoltageContext *context;
    char *error_message;
    long num_bytes_per_cc_per_timestep;
    unsigned long gps_second_start;
    size_t gps_second_count;
    int coarse_chan_index;
    long local_sum;
    int version;
} ThreadArgs_read_second;

void *process_coarse_channel_read_second(void *arg)
{
    ThreadArgs_read_second *args = (ThreadArgs_read_second *)arg;

    signed char *buffer = calloc(args->num_bytes_per_cc_per_timestep, sizeof(signed char));
    if (!buffer)
    {
        perror("calloc");
        pthread_exit(NULL);
    }

    int ret = -1;

    if (args->version == 1)
    {
        ret = mwalib_voltage_context_read_second(args->context,
                                                 args->gps_second_start,
                                                 args->gps_second_count,
                                                 args->coarse_chan_index,
                                                 buffer,
                                                 args->num_bytes_per_cc_per_timestep,
                                                 args->error_message,
                                                 ERROR_MESSAGE_LEN);
    }
    else if (args->version == 2)
    {
        ret = mwalib_voltage_context_read_second2(args->context,
                                                  args->gps_second_start,
                                                  args->gps_second_count,
                                                  args->coarse_chan_index,
                                                  buffer,
                                                  args->num_bytes_per_cc_per_timestep,
                                                  args->error_message,
                                                  ERROR_MESSAGE_LEN);
    }
    else
    {
        printf("Invalid version specified: %d", args->version);
        free(buffer);
        pthread_exit((void *)-1);
    }

    if (ret == MWALIB_SUCCESS)
    {
        long sum = 0;
        for (long i = 0; i < args->num_bytes_per_cc_per_timestep; i++)
        {
            sum += buffer[i];
        }
        args->local_sum = sum;
    }
    else if (ret == MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN)
    {
        args->local_sum = 0;
    }
    else
    {
        printf("Error: %s\n", args->error_message);
        free(buffer);
        pthread_exit((void *)-1);
    }

    free(buffer);
    pthread_exit(NULL);
}

void *process_coarse_channel_read_file(void *arg)
{
    ThreadArgs_read_file *args = (ThreadArgs_read_file *)arg;

    signed char *buffer = calloc(args->num_bytes_per_cc_per_timestep, sizeof(signed char));
    if (!buffer)
    {
        perror("calloc");
        pthread_exit(NULL);
    }

    int ret = -1;

    if (args->version == 1)
    {
        ret = mwalib_voltage_context_read_file(args->context,
                                               args->timestep_index,
                                               args->coarse_chan_index,
                                               buffer,
                                               args->num_bytes_per_cc_per_timestep,
                                               args->error_message,
                                               ERROR_MESSAGE_LEN);
    }
    else if (args->version == 2)
    {
        ret = mwalib_voltage_context_read_file2(args->context,
                                                args->timestep_index,
                                                args->coarse_chan_index,
                                                buffer,
                                                args->num_bytes_per_cc_per_timestep,
                                                args->error_message,
                                                ERROR_MESSAGE_LEN);
    }
    else
    {
        printf("Invalid version specified: %d", args->version);
        free(buffer);
        pthread_exit((void *)-1);
    }

    if (ret == MWALIB_SUCCESS)
    {
        long sum = 0;
        for (long i = 0; i < args->num_bytes_per_cc_per_timestep; i++)
        {
            sum += buffer[i];
        }
        args->local_sum = sum;
    }
    else if (ret == MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN)
    {
        args->local_sum = 0;
    }
    else
    {
        printf("Error: %s\n", args->error_message);
        free(buffer);
        pthread_exit((void *)-1);
    }

    free(buffer);
    pthread_exit(NULL);
}

void do_sum_parallel_pthreads_read_file(VoltageContext *context,
                                        long num_bytes_per_cc_per_timestep,
                                        unsigned int first_timestep_index,
                                        unsigned int last_timestep_index,
                                        unsigned int first_chan_index,
                                        unsigned int last_chan_index, int version)
{
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));
    int num_coarse_chans = last_chan_index - first_chan_index + 1;
    long total_sum = 0;

    for (unsigned int timestep_index = first_timestep_index; timestep_index <= last_timestep_index; timestep_index++)
    {
        pthread_t threads[num_coarse_chans];
        ThreadArgs_read_file args[num_coarse_chans];

        // Create threads — one per coarse channel
        for (unsigned int coarse_chan_index = first_chan_index; coarse_chan_index <= last_chan_index; coarse_chan_index++)
        {
            int arg_index = coarse_chan_index - first_chan_index;
            args[arg_index].context = context;
            args[arg_index].error_message = error_message;
            args[arg_index].num_bytes_per_cc_per_timestep = num_bytes_per_cc_per_timestep;
            args[arg_index].timestep_index = timestep_index;
            args[arg_index].coarse_chan_index = coarse_chan_index;
            args[arg_index].local_sum = 0;
            args[arg_index].version = version;

            printf("Timestep index: %d, Coarse channel index: %d\n",
                   timestep_index, coarse_chan_index);

            if (pthread_create(&threads[arg_index], NULL,
                               process_coarse_channel_read_file, &args[arg_index]) != 0)
            {
                perror("pthread_create");
                exit(EXIT_FAILURE);
            }
        }

        // Join threads and accumulate timestep sum
        long timestep_sum = 0;
        for (int thread_index = 0; thread_index < num_coarse_chans; thread_index++)
        {
            pthread_join(threads[thread_index], NULL);
            timestep_sum += args[thread_index].local_sum;
        }

        total_sum += timestep_sum;
    }

    printf("Total sum: %ld\n", total_sum);
    free(error_message);
}

void do_sum_parallel_pthreads_read_second(VoltageContext *context,
                                          long num_bytes_per_cc_per_timestep,
                                          unsigned int first_timestep_index,
                                          unsigned int last_timestep_index,
                                          unsigned int first_gps_second,
                                          unsigned int timestep_duration_seconds,
                                          unsigned int first_chan_index,
                                          unsigned int last_chan_index, int version)
{
    char *error_message = malloc(ERROR_MESSAGE_LEN * sizeof(char));
    int num_coarse_chans = last_chan_index - first_chan_index + 1;
    int num_timesteps = last_timestep_index - first_timestep_index + 1;
    long total_sum = 0;

    for (int timestep_index = 0; timestep_index < num_timesteps; timestep_index++)
    {
        pthread_t threads[num_coarse_chans];
        ThreadArgs_read_second args[num_coarse_chans];

        unsigned long gps_second_start = first_gps_second + (timestep_index * timestep_duration_seconds);

        // Create threads — one per coarse channel
        for (unsigned int coarse_chan_index = first_chan_index; coarse_chan_index <= last_chan_index; coarse_chan_index++)
        {
            int arg_index = coarse_chan_index - first_chan_index;
            args[arg_index].context = context;
            args[arg_index].error_message = error_message;
            args[arg_index].num_bytes_per_cc_per_timestep = num_bytes_per_cc_per_timestep;
            args[arg_index].gps_second_start = gps_second_start;
            args[arg_index].gps_second_count = timestep_duration_seconds;
            args[arg_index].coarse_chan_index = coarse_chan_index;
            args[arg_index].local_sum = 0;
            args[arg_index].version = version;

            printf("GPS second start: %lu, Coarse channel index: %d\n",
                   gps_second_start, coarse_chan_index);
            if (pthread_create(&threads[arg_index], NULL,
                               process_coarse_channel_read_second, &args[arg_index]) != 0)
            {
                perror("pthread_create");
                exit(EXIT_FAILURE);
            }
        }

        // Join threads and accumulate timestep sum
        long timestep_sum = 0;
        for (int thread_index = 0; thread_index < num_coarse_chans; thread_index++)
        {
            pthread_join(threads[thread_index], NULL);
            timestep_sum += args[thread_index].local_sum;
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

    int num_timesteps = voltage_metadata->num_common_timesteps;
    int num_coarse_chans = voltage_metadata->num_common_coarse_chans;
    long num_bytes_per_cc_per_ts = voltage_metadata->num_voltage_blocks_per_timestep * voltage_metadata->voltage_block_size_bytes;
    size_t timestep_duration = voltage_metadata->timestep_duration_ms / 1000;
    unsigned long first_timestep_index = voltage_metadata->common_timestep_indices[0];
    unsigned long last_timestep_index = voltage_metadata->common_timestep_indices[voltage_metadata->num_common_timesteps - 1];
    unsigned long first_gps_second = voltage_metadata->timesteps[first_timestep_index].gps_time_ms / 1000;
    unsigned long last_gps_second = (voltage_metadata->timesteps[last_timestep_index].gps_time_ms / 1000) + timestep_duration - 1;
    unsigned long first_cc_index = voltage_metadata->common_coarse_chan_indices[0];
    unsigned long last_cc_index = voltage_metadata->common_coarse_chan_indices[voltage_metadata->num_common_coarse_chans - 1];
    unsigned long first_cc_number = voltage_metadata->coarse_chans[first_cc_index].rec_chan_number;
    unsigned long last_cc_number = voltage_metadata->coarse_chans[last_cc_index].rec_chan_number;

    printf("Number of timesteps: %d\n", num_timesteps);
    printf("..First GPS second: %lu [%lu]\n", first_gps_second, first_timestep_index);
    printf("..Last GPS second: %lu [%lu]\n", last_gps_second, last_timestep_index);
    printf("Number of coarse channels: %d\n", num_coarse_chans);
    printf("..First coarse channel: %lu [%lu]\n", first_cc_number, first_cc_index);
    printf("..Last coarse channel: %lu [%lu]\n", last_cc_number, last_cc_index);
    printf("Number of bytes per coarse channel per timestep: %ld\n", num_bytes_per_cc_per_ts);
    printf("Timestep duration (seconds): %zu\n", timestep_duration);

    // Setup timings
    struct timeval start, end;
    gettimeofday(&start, NULL);

    printf("Running sum using mwalib_voltage_context_read_file...\n");
    gettimeofday(&start, NULL);
    do_sum_parallel_pthreads_read_file(voltage_context, num_bytes_per_cc_per_ts, first_timestep_index, last_timestep_index, first_cc_index, last_cc_index, 1);
    gettimeofday(&end, NULL);
    double elapsed = (end.tv_sec - start.tv_sec) +
                     (end.tv_usec - start.tv_usec) / 1e6;
    printf("Elapsed time: %.6f seconds\n", elapsed);

    printf("Running sum using mwalib_voltage_context_read_file2...\n");
    gettimeofday(&start, NULL);
    do_sum_parallel_pthreads_read_file(voltage_context, num_bytes_per_cc_per_ts, first_timestep_index, last_timestep_index, first_cc_index, last_cc_index, 2);
    gettimeofday(&end, NULL);
    elapsed = (end.tv_sec - start.tv_sec) +
              (end.tv_usec - start.tv_usec) / 1e6;
    printf("Elapsed time: %.6f seconds\n", elapsed);

    printf("Running sum using mwalib_voltage_context_read_second...\n");
    gettimeofday(&start, NULL);
    do_sum_parallel_pthreads_read_second(voltage_context, num_bytes_per_cc_per_ts, first_timestep_index, last_timestep_index, first_gps_second, timestep_duration, first_cc_index, last_cc_index, 1);
    gettimeofday(&end, NULL);
    elapsed = (end.tv_sec - start.tv_sec) +
              (end.tv_usec - start.tv_usec) / 1e6;
    printf("Elapsed time: %.6f seconds\n", elapsed);

    printf("Running sum using mwalib_voltage_context_read_second2...\n");
    gettimeofday(&start, NULL);
    do_sum_parallel_pthreads_read_second(voltage_context, num_bytes_per_cc_per_ts, first_timestep_index, last_timestep_index, first_gps_second, timestep_duration, first_cc_index, last_cc_index, 2);
    gettimeofday(&end, NULL);
    elapsed = (end.tv_sec - start.tv_sec) +
              (end.tv_usec - start.tv_usec) / 1e6;
    printf("Elapsed time: %.6f seconds\n", elapsed);

    mwalib_metafits_metadata_free(metafits_metadata);
    mwalib_voltage_metadata_free(voltage_metadata);
    mwalib_voltage_context_free(voltage_context);
    free(volt_files);
    free(error_message);

    return EXIT_SUCCESS;
}