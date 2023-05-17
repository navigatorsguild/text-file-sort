use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Error};
use benchmark_rs::benchmarks::Benchmarks;
use benchmark_rs::stopwatch::StopWatch;
use data_encoding::HEXLOWER;
use simple_logger::SimpleLogger;

use text_file_sort::sort::Sort;

use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Clone)]
pub struct BenchmarkConfig {
    files: BTreeMap<usize, PathBuf>,
    bench_results_dir: PathBuf,
    bench_tmp_dir: PathBuf,
    tasks: usize,
    concurrent_merge: bool,
    chunk_size_bytes: u64,
    intermediate: usize,
    description: String,
}

impl BenchmarkConfig {
    pub fn new(files: BTreeMap<usize, PathBuf>, bench_results_dir: PathBuf, bench_tmp_dir: PathBuf, tasks: usize, concurrent_merge: bool, chunk_size_bytes: u64, intermediate: usize, description: &str) -> BenchmarkConfig {
        BenchmarkConfig {
            files,
            bench_results_dir,
            bench_tmp_dir,
            tasks,
            concurrent_merge,
            chunk_size_bytes,
            intermediate,
            description: description.to_string(),
        }
    }

    pub fn get_input_path(&self, key: usize) -> PathBuf {
        self.files.get(&key).unwrap().clone()
    }

    pub fn get_input_paths(&self) -> Vec<PathBuf> {
        self.files.values().cloned().collect()
    }

    pub fn bench_results_dir(&self) -> &PathBuf {
        &self.bench_results_dir
    }

    pub fn bench_tmp_dir(&self) -> &PathBuf {
        &self.bench_tmp_dir
    }

    pub fn tasks(&self) -> usize {
        self.tasks
    }

    pub fn concurrent_merge(&self) -> bool {
        self.concurrent_merge
    }

    pub fn chunk_size_bytes(&self) -> u64 {
        self.chunk_size_bytes
    }

    pub fn intermediate(&self) -> usize {
        self.intermediate
    }
}

impl Display for BenchmarkConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tasks: {}, intermediate: {}, description: {}",
                 self.tasks,
                 self.intermediate,
                 self.description,
        )
    }
}

fn temp_file_name(dir: &PathBuf) -> PathBuf {
    let mut result = PathBuf::from(dir);
    let name = HEXLOWER.encode(&rand::random::<[u8; 16]>());
    result.push(name);
    result
}

fn cleanup(bench_results_dir: &PathBuf) -> Result<(), anyhow::Error> {
    if bench_results_dir.exists() {
        fs::remove_dir_all(bench_results_dir.clone()).with_context(|| anyhow!("{}", bench_results_dir.to_string_lossy()))?;
    }
    Ok(())
}

fn setup(bench_input_dir: &PathBuf, bench_results_dir: &PathBuf, bench_tmp_dir: &PathBuf) -> Result<(), anyhow::Error> {
    cleanup(bench_results_dir)?;

    if !bench_input_dir.exists() {
        fs::create_dir_all(bench_input_dir.clone())?;
    }

    if !bench_results_dir.exists() {
        fs::create_dir_all(bench_results_dir.clone())
            .with_context(|| anyhow!("{}", bench_results_dir.to_string_lossy()))?;
    }

    if !bench_tmp_dir.exists() {
        fs::create_dir_all(bench_tmp_dir.clone())
            .with_context(|| anyhow!("{}", bench_tmp_dir.to_string_lossy()))?;
    }

    Ok(())
}

fn create_input_files(count: usize, factor: usize, seed_size: usize, seed_path: PathBuf, base_path: PathBuf) -> Result<BTreeMap<usize, PathBuf>, anyhow::Error> {
    let mut files: BTreeMap<usize, PathBuf> = BTreeMap::new();
    let seed_content = fs::read_to_string(&seed_path)
        .with_context(|| anyhow!("path: {}", seed_path.to_string_lossy()))?;
    for i in 1..=count {
        let number_of_lines = i * factor * seed_size;
        let path = base_path.join(PathBuf::from(number_of_lines.to_string()));
        if !path.exists() {
            let mut writer = BufWriter::new(
                File::create(&path)
                    .with_context(|| anyhow!("path: {}", path.to_string_lossy()))?);
            for _j in 0..(i * factor) {
                writer.write_all(seed_content.as_bytes())?;
            }
        }
        files.insert(number_of_lines, path);
    }
    Ok(files)
}

fn sort(stop_watch: &mut StopWatch, config: BenchmarkConfig, work: usize) -> Result<(), anyhow::Error> {
    stop_watch.pause();
    let input_path = config.get_input_path(work);
    let output_path = temp_file_name(config.bench_results_dir());
    log::info!("Start sorting {}", input_path.to_string_lossy());
    stop_watch.resume();
    let mut text_file_sort = Sort::new(vec![input_path.clone()], output_path.clone());
    text_file_sort.with_tmp_dir(config.bench_tmp_dir().clone());
    text_file_sort.with_tasks(config.tasks());
    text_file_sort.with_concurrent_merge(config.concurrent_merge());
    text_file_sort.with_chunk_size_bytes(config.chunk_size_bytes());
    text_file_sort.with_intermediate_files(config.intermediate());
    text_file_sort.sort()?;
    stop_watch.pause();
    log::info!("Finish sorting {}", input_path.to_string_lossy());
    fs::remove_file(output_path.clone())
        .with_context(|| anyhow!("{}", output_path.to_string_lossy()))?;
    Ok(())
}

#[test]
fn text_file_sort_bench() -> Result<(), Error> {
    SimpleLogger::new().init().unwrap();
    log::info!("Started text_file_sort_bench.");

    let bench_input_dir = PathBuf::from("./target/benchmarks/input");
    let bench_results_dir = PathBuf::from("./target/benchmarks/results");
    let bench_tmp_dir = PathBuf::from("./target/benchmarks/results/tmp");
    let seed_path = PathBuf::from("./tests/fixtures/sorted-10000.dat");
    setup(&bench_input_dir, &bench_results_dir, &bench_tmp_dir)?;

    let small_files = create_input_files(20, 10, 10_000, seed_path.clone(), bench_input_dir.clone())?;
    let medium_files = create_input_files(20, 100, 10_000, seed_path.clone(), bench_input_dir.clone())?;
    let large_files = create_input_files(20, 1000, 10_000, seed_path.clone(), bench_input_dir.clone())?;

    let mut benchmarks = Benchmarks::new("text-file-sort");

    // small files
    benchmarks.add(
        "small-files-1-tasks",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            1,
            false,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-1-tasks-cm",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            1,
            true,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-2-tasks",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            2,
            false,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-2-tasks-cm",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            2,
            true,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-4-tasks",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            4,
            false,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-4-tasks-cm",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            4,
            true,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-8-tasks",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            8,
            false,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "small-files-8-tasks-cm",
        sort,
        BenchmarkConfig::new(
            small_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            8,
            true,
            100_000_000,
            8192,
            "small files",
        ),
        small_files.keys().cloned().collect(),
        3,
        0,
    )?;

    // medium files
    benchmarks.add(
        "medium-files-1-tasks",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            1,
            false,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-1-tasks-cm",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            1,
            true,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-2-tasks",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            2,
            false,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-2-tasks-cm",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            2,
            true,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-4-tasks",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            4,
            false,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-4-tasks-cm",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            4,
            true,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-8-tasks",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            8,
            false,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "medium-files-8-tasks-cm",
        sort,
        BenchmarkConfig::new(
            medium_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            8,
            true,
            100_000_000,
            8192,
            "medium files",
        ),
        medium_files.keys().cloned().collect(),
        3,
        0,
    )?;

    // large files
    benchmarks.add(
        "large-files-1-tasks",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            1,
            false,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-1-tasks-cm",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            1,
            true,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-2-tasks",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            2,
            false,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-2-tasks-cm",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            2,
            true,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-4-tasks",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            4,
            false,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-4-tasks-cm",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            4,
            true,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-8-tasks",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            8,
            false,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.add(
        "large-files-8-tasks-cm",
        sort,
        BenchmarkConfig::new(
            large_files.clone(),
            bench_results_dir.clone(),
            bench_tmp_dir.clone(),
            8,
            true,
            100_000_000,
            8192,
            "large files",
        ),
        large_files.keys().cloned().collect(),
        3,
        0,
    )?;

    benchmarks.run()?;
    benchmarks.save_to_csv(PathBuf::from("./target/benchmarks/"), true, true)?;
    benchmarks.save_to_json(PathBuf::from("./target/benchmarks/"))?;

    log::info!("Finished text_file_sort_bench.");
    Ok(())
}
