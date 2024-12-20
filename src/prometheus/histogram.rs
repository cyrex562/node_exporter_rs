const NATIVE_HISTOGRAM_SCHEMA_MAXIMUM: i32 = 8;
const NATIVE_HISTOGRAM_SCHEMA_MINIMUM: i32 = -4;

const NATIVE_HISTOGRAM_BOUNDS: &[&[f64]] = &[
    &[0.5],
    &[0.5, 0.7071067811865475],
    &[
        0.5,
        0.5946035575013605,
        0.7071067811865475,
        0.8408964152537144,
    ],
    &[
        0.5,
        0.5452538663326288,
        0.5946035575013605,
        0.6484197773255048,
        0.7071067811865475,
        0.7711054127039704,
        0.8408964152537144,
        0.9170040432046711,
    ],
    &[
        0.5,
        0.5221368912137069,
        0.5452538663326288,
        0.5693943173783458,
        0.5946035575013605,
        0.620928906036742,
        0.6484197773255048,
        0.6771277734684463,
        0.7071067811865475,
        0.7384130729697496,
        0.7711054127039704,
        0.805245165974627,
        0.8408964152537144,
        0.8781260801866495,
        0.9170040432046711,
        0.9576032806985735,
    ],
    &[
        0.5,
        0.5109485743270583,
        0.5221368912137069,
        0.5335702003384117,
        0.5452538663326288,
        0.5571933712979462,
        0.5693943173783458,
        0.5818624293887887,
        0.5946035575013605,
        0.6076236799902344,
        0.620928906036742,
        0.6345254785958666,
        0.6484197773255048,
        0.6626183215798706,
        0.6771277734684463,
        0.6919549409819159,
        0.7071067811865475,
        0.7225904034885232,
        0.7384130729697496,
        0.7545822137967112,
        0.7711054127039704,
        0.7879904225539431,
        0.805245165974627,
        0.8228777390769823,
        0.8408964152537144,
        0.8593096490612387,
        0.8781260801866495,
        0.8973545375015533,
        0.9170040432046711,
        0.9370838170551498,
        0.9576032806985735,
        0.9785720620876999,
    ],
    &[
        0.5,
        0.5054446430258502,
        0.5109485743270583,
        0.5165124395106142,
        0.5221368912137069,
        0.5278225891802786,
        0.5335702003384117,
        0.5393803988785598,
        0.5452538663326288,
        0.5511912916539204,
        0.5571933712979462,
        0.5632608093041209,
        0.5693943173783458,
        0.5755946149764913,
        0.5818624293887887,
        0.5881984958251406,
        0.5946035575013605,
        0.6010783657263515,
        0.6076236799902344,
        0.6142402680534349,
        0.620928906036742,
        0.6276903785123455,
        0.6345254785958666,
        0.6414350080393891,
        0.6484197773255048,
        0.6554806057623822,
        0.6626183215798706,
        0.6698337620266515,
        0.6771277734684463,
        0.6845012114872953,
        0.6919549409819159,
        0.6994898362691555,
        0.7071067811865475,
        0.7148066691959849,
        0.7225904034885232,
        0.7304588970903234,
        0.7384130729697496,
        0.7464538641456323,
        0.7545822137967112,
        0.762799075372269,
        0.7711054127039704,
        0.7795022001189185,
        0.7879904225539431,
        0.7965710756711334,
        0.805245165974627,
        0.8140137109286738,
        0.8228777390769823,
        0.8318382901633681,
        0.8408964152537144,
        0.8500531768592616,
        0.8593096490612387,
        0.8686669176368529,
        0.8781260801866495,
        0.8876882462632604,
        0.8973545375015533,
        0.9071260877501991,
        0.9170040432046711,
        0.9269895625416926,
        0.9370838170551498,
        0.9472879907934827,
        0.9576032806985735,
        0.9680308967461471,
        0.9785720620876999,
        0.9892280131939752,
    ],
    &[
        0.5,
        0.5027149505564014,
        0.5054446430258502,
        0.5081891574554764,
        0.5109485743270583,
        0.5137229745593818,
        0.5165124395106142,
        0.5193170509806894,
        0.5221368912137069,
        0.5249720429003435,
        0.5278225891802786,
        0.5306886136446309,
        0.5335702003384117,
        0.5364674337629877,
        0.5393803988785598,
        0.5423091811066545,
        0.5452538663326288,
        0.5482145409081883,
        0.5511912916539204,
        0.5541842058618393,
        0.5571933712979462,
        0.5602188762048033,
        0.5632608093041209,
        0.5663192597993595,
        0.5693943173783458,
        0.572486072215902,
        0.5755946149764913,
        0.5787200368168754,
        0.5818624293887887,
        0.585021884841625,
        0.5881984958251406,
        0.5913923554921704,
        0.5946035575013605,
        0.5978321960199137,
        0.6010783657263515,
        0.6043421618132907,
        0.6076236799902344,
        0.6109230164863786,
        0.6142402680534349,
        0.6175755319684665,
        0.620928906036742,
        0.6243004885946023,
        0.6276903785123455,
        0.6310986751971253,
        0.6345254785958666,
        0.637970889198196,
        0.6414350080393891,
        0.6449179367033329,
        0.6484197773255048,
        0.6519406325959679,
        0.6554806057623822,
        0.659039800633032,
        0.6626183215798706,
        0.6662162735415805,
        0.6698337620266515,
        0.6734708931164728,
        0.6771277734684463,
        0.6808045103191123,
        0.6845012114872953,
        0.688217985377265,
        0.6919549409819159,
        0.6957121878859629,
        0.6994898362691555,
        0.7032879969095076,
        0.7071067811865475,
        0.7109463010845827,
        0.7148066691959849,
        0.718687998724491,
        0.7225904034885232,
        0.7265139979245261,
        0.7304588970903234,
        0.7344252166684908,
        0.7384130729697496,
        0.7424225829363761,
        0.7464538641456323,
        0.7505070348132126,
        0.7545822137967112,
        0.7586795205991071,
        0.762799075372269,
        0.7669409989204777,
        0.7711054127039704,
        0.7752924388424999,
        0.7795022001189185,
        0.7837348199827764,
        0.7879904225539431,
        0.7922691326262467,
        0.7965710756711334,
        0.8008963778413465,
        0.805245165974627,
        0.8096175675974316,
        0.8140137109286738,
        0.8184337248834821,
        0.8228777390769823,
        0.8273458838280969,
        0.8318382901633681,
        0.8363550898207981,
        0.8408964152537144,
        0.8454623996346523,
        0.8500531768592616,
        0.8546688815502312,
        0.8593096490612387,
        0.8639756154809185,
        0.8686669176368529,
        0.8733836930995842,
        0.8781260801866495,
        0.8828942179666361,
        0.8876882462632604,
        0.8925083056594671,
        0.8973545375015533,
        0.9022270839033115,
        0.9071260877501991,
        0.9120516927035263,
        0.9170040432046711,
        0.9219832844793128,
        0.9269895625416926,
        0.9320230241988943,
        0.9370838170551498,
        0.9421720895161669,
        0.9472879907934827,
        0.9524316709088368,
        0.9576032806985735,
        0.9628029718180622,
        0.9680308967461471,
        0.9732872087896164,
        0.9785720620876999,
        0.9838856116165875,
        0.9892280131939752,
        0.9945994234836328,
    ],
    &[
        0.5,
        0.5013556375251013,
        0.5027149505564014,
        0.5040779490592088,
        0.5054446430258502,
        0.5068150424757447,
        0.5081891574554764,
        0.509566998038869,
        0.5109485743270583,
        0.5123338964485679,
        0.5137229745593818,
        0.5151158188430205,
        0.5165124395106142,
        0.5179128468009786,
        0.5193170509806894,
        0.520725062344158,
        0.5221368912137069,
        0.5235525479396449,
        0.5249720429003435,
        0.526395386502313,
        0.5278225891802786,
        0.5292536613972564,
        0.5306886136446309,
        0.5321274564422321,
        0.5335702003384117,
        0.5350168559101208,
        0.5364674337629877,
        0.5379219445313954,
        0.5393803988785598,
        0.5408428074966075,
        0.5423091811066545,
        0.5437795304588847,
        0.5452538663326288,
        0.5467321995364429,
        0.5482145409081883,
        0.549700901315111,
        0.5511912916539204,
        0.5526857228508706,
        0.5541842058618393,
        0.5556867516724088,
        0.5571933712979462,
        0.5587040757836845,
        0.5602188762048033,
        0.5617377836665098,
        0.5632608093041209,
        0.564787964283144,
        0.5663192597993595,
        0.5678547070789026,
        0.5693943173783458,
        0.5709381019847808,
        0.572486072215902,
        0.5740382394200894,
        0.5755946149764913,
        0.5771552102951081,
        0.5787200368168754,
        0.5802891060137493,
        0.5818624293887887,
        0.5834400184762408,
        0.585021884841625,
        0.5866080400818185,
        0.5881984958251406,
        0.5897932637314379,
        0.5913923554921704,
        0.5929957828304968,
        0.5946035575013605,
        0.5962156912915756,
        0.5978321960199137,
        0.5994530835371903,
        0.6010783657263515,
        0.6027080545025619,
        0.6043421618132907,
        0.6059806996384005,
        0.6076236799902344,
        0.6092711149137041,
        0.6109230164863786,
        0.6125793968185725,
        0.6142402680534349,
        0.6159056423670379,
        0.6175755319684665,
        0.6192499490999082,
        0.620928906036742,
        0.622612415087629,
        0.6243004885946023,
        0.6259931389331581,
        0.6276903785123455,
        0.6293922197748583,
        0.6310986751971253,
        0.6328097572894031,
        0.6345254785958666,
        0.6362458516947014,
        0.637970889198196,
        0.6397006037528346,
        0.6414350080393891,
        0.6431741147730128,
        0.6449179367033329,
        0.6466664866145447,
        0.6484197773255048,
        0.6501778216898253,
        0.6519406325959679,
        0.6537082229673385,
        0.6554806057623822,
        0.6572577939746774,
        0.659039800633032,
        0.6608266388015788,
        0.6626183215798706,
        0.6644148621029772,
        0.6662162735415805,
        0.6680225691020727,
        0.6698337620266515,
        0.6716498655934177,
        0.6734708931164728,
        0.6752968579460171,
        0.6771277734684463,
        0.6789636531064505,
        0.6808045103191123,
        0.6826503586020058,
        0.6845012114872953,
        0.6863570825438342,
        0.688217985377265,
        0.690083933630119,
        0.6919549409819159,
        0.6938310211492645,
        0.6957121878859629,
        0.6975984549830999,
        0.6994898362691555,
        0.7013863456101023,
        0.7032879969095076,
        0.7051948041086352,
        0.7071067811865475,
        0.7090239421602076,
        0.7109463010845827,
        0.7128738720527471,
        0.7148066691959849,
        0.7167447066838943,
        0.718687998724491,
        0.7206365595643126,
        0.7225904034885232,
        0.7245495448210174,
        0.7265139979245261,
        0.7284837772007218,
        0.7304588970903234,
        0.7324393720732029,
        0.7344252166684908,
        0.7364164454346837,
        0.7384130729697496,
        0.7404151139112358,
        0.7424225829363761,
        0.7444354947621984,
        0.7464538641456323,
        0.7484777058836176,
        0.7505070348132126,
        0.7525418658117031,
        0.7545822137967112,
        0.7566280937263048,
        0.7586795205991071,
        0.7607365094544071,
        0.762799075372269,
        0.7648672334736434,
        0.7669409989204777,
        0.7690203869158282,
        0.7711054127039704,
        0.7731960915705107,
        0.7752924388424999,
        0.7773944698885442,
        0.7795022001189185,
        0.7816156449856788,
        0.7837348199827764,
        0.7858597406461707,
        0.7879904225539431,
        0.7901268813264122,
        0.7922691326262467,
        0.7944171921585818,
        0.7965710756711334,
        0.7987307989543135,
        0.8008963778413465,
        0.8030678282083853,
        0.805245165974627,
        0.8074284071024302,
        0.8096175675974316,
        0.8118126635086642,
        0.8140137109286738,
        0.8162207259936375,
        0.8184337248834821,
        0.820652723822003,
        0.8228777390769823,
        0.8251087869603088,
        0.8273458838280969,
        0.8295890460808079,
        0.8318382901633681,
        0.8340936325652911,
        0.8363550898207981,
        0.8386226785089391,
        0.8408964152537144,
        0.8431763167241966,
        0.8454623996346523,
        0.8477546807446661,
        0.8500531768592616,
        0.8523579048290255,
        0.8546688815502312,
        0.8569861239649629,
        0.8593096490612387,
        0.8616394738731368,
        0.8639756154809185,
        0.8663180910111553,
        0.8686669176368529,
        0.871022112577578,
        0.8733836930995842,
        0.8757516765159389,
        0.8781260801866495,
        0.8805069215187917,
        0.8828942179666361,
        0.8852879870317771,
        0.8876882462632604,
        0.890095013257712,
        0.8925083056594671,
        0.8949281411607002,
        0.8973545375015533,
        0.8997875124702672,
        0.9022270839033115,
        0.9046732696855155,
        0.9071260877501991,
        0.909585556079304,
        0.9120516927035263,
        0.9145245157024483,
        0.9170040432046711,
        0.9194902933879467,
        0.9219832844793128,
        0.9244830347552253,
        0.9269895625416926,
        0.92950288621441,
        0.9320230241988943,
        0.9345499949706191,
        0.9370838170551498,
        0.93962450902828,
        0.9421720895161669,
        0.9447265771954693,
        0.9472879907934827,
        0.9498563490882775,
        0.9524316709088368,
        0.9550139751351947,
        0.9576032806985735,
        0.9601996065815236,
        0.9628029718180622,
        0.9654133954938133,
        0.9680308967461471,
        0.9706554947643201,
        0.9732872087896164,
        0.9759260581154889,
        0.9785720620876999,
        0.9812252401044634,
        0.9838856116165875,
        0.9865531961276168,
        0.9892280131939752,
        0.9919100824251095,
        0.9945994234836328,
        0.9972960560854698,
    ],
];

pub trait Histogram: Metric + Collector {
    /// Observe adds a single observation to the histogram. Observations are
    /// usually positive or zero. Negative observations are accepted but
    /// prevent current versions of Prometheus from properly detecting
    /// counter resets in the sum of observations. (The experimental Native
    /// Histograms handle negative observations properly.) See
    /// https://prometheus.io/docs/practices/histograms/#count-and-sum-of-observations
    /// for details.
    fn observe(&self, value: f64);
}

const BUCKET_LABEL: &str = "le";

// DefBuckets are the default Histogram buckets. The default buckets are
// tailored to broadly measure the response time (in seconds) of a network
// service. Most likely, however, you will be required to define buckets
// customized to your use case.
pub const DEF_BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

// DefNativeHistogramZeroThreshold is the default value for
// NativeHistogramZeroThreshold in the HistogramOpts.
//
// The value is 2^-128 (or 0.5*2^-127 in the actual IEEE 754 representation),
// which is a bucket boundary at all possible resolutions.
pub const DEF_NATIVE_HISTOGRAM_ZERO_THRESHOLD: f64 = 2.938735877055719e-39;

// NativeHistogramZeroThresholdZero can be used as NativeHistogramZeroThreshold
// in the HistogramOpts to create a zero bucket of width zero, i.e. a zero
// bucket that only receives observations of precisely zero.
pub const NATIVE_HISTOGRAM_ZERO_THRESHOLD_ZERO: f64 = -1.0;

pub const ERR_BUCKET_LABEL_NOT_ALLOWED: &str = "\"le\" is not allowed as label name in histograms";

pub fn linear_buckets(start: f64, width: f64, count: usize) -> Vec<f64> {
    if count < 1 {
        panic!("linear_buckets needs a positive count");
    }
    let mut buckets = Vec::with_capacity(count);
    let mut current = start;
    for _ in 0..count {
        buckets.push(current);
        current += width;
    }
    buckets
}

pub fn exponential_buckets(start: f64, factor: f64, count: usize) -> Vec<f64> {
    if count < 1 {
        panic!("exponential_buckets needs a positive count");
    }
    if start <= 0.0 {
        panic!("exponential_buckets needs a positive start value");
    }
    if factor <= 1.0 {
        panic!("exponential_buckets needs a factor greater than 1");
    }
    let mut buckets = Vec::with_capacity(count);
    let mut current = start;
    for _ in 0..count {
        buckets.push(current);
        current *= factor;
    }
    buckets
}

pub fn exponential_buckets_range(min_bucket: f64, max_bucket: f64, count: usize) -> Vec<f64> {
    if count < 1 {
        panic!("exponential_buckets_range count needs a positive count");
    }
    if min_bucket <= 0.0 {
        panic!("exponential_buckets_range min needs to be greater than 0");
    }

    let growth_factor = (max_bucket / min_bucket).powf(1.0 / (count - 1) as f64);

    let mut buckets = Vec::with_capacity(count);
    for i in 0..count {
        buckets.push(min_bucket * growth_factor.powf(i as f64));
    }
    buckets
}

use std::collections::HashMap;
use std::time::Duration;

pub struct HistogramOpts {
    // Namespace, Subsystem, and Name are components of the fully-qualified
    // name of the Histogram (created by joining these components with
    // "_"). Only Name is mandatory, the others merely help structuring the
    // name. Note that the fully-qualified name of the Histogram must be a
    // valid Prometheus metric name.
    pub namespace: String,
    pub subsystem: String,
    pub name: String,

    // Help provides information about this Histogram.
    //
    // Metrics with the same fully-qualified name must have the same Help
    // string.
    pub help: String,

    // ConstLabels are used to attach fixed labels to this metric. Metrics
    // with the same fully-qualified name must have the same label names in
    // their ConstLabels.
    //
    // ConstLabels are only used rarely. In particular, do not use them to
    // attach the same labels to all your metrics. Those use cases are
    // better covered by target labels set by the scraping Prometheus
    // server, or by one specific metric (e.g. a build_info or a
    // machine_role metric). See also
    // https://prometheus.io/docs/instrumenting/writing_exporters/#target-labels-not-static-scraped-labels
    pub const_labels: HashMap<String, String>,

    // Buckets defines the buckets into which observations are counted. Each
    // element in the slice is the upper inclusive bound of a bucket. The
    // values must be sorted in strictly increasing order. There is no need
    // to add a highest bucket with +Inf bound, it will be added
    // implicitly. If Buckets is left as None or set to an empty vector,
    // it is replaced by default buckets. The default buckets are
    // DEF_BUCKETS if no buckets for a native histogram (see below) are used,
    // otherwise the default is no buckets. (In other words, if you want to
    // use both regular buckets and buckets for a native histogram, you have
    // to define the regular buckets here explicitly.)
    pub buckets: Option<Vec<f64>>,

    // If native_histogram_bucket_factor is greater than one, so-called sparse
    // buckets are used (in addition to the regular buckets, if defined
    // above). A Histogram with sparse buckets will be ingested as a Native
    // Histogram by a Prometheus server with that feature enabled (requires
    // Prometheus v2.40+). Sparse buckets are exponential buckets covering
    // the whole float64 range (with the exception of the “zero” bucket, see
    // native_histogram_zero_threshold below). From any one bucket to the next,
    // the width of the bucket grows by a constant
    // factor. native_histogram_bucket_factor provides an upper bound for this
    // factor (exception see below). The smaller
    // native_histogram_bucket_factor, the more buckets will be used and thus
    // the more costly the histogram will become. A generally good trade-off
    // between cost and accuracy is a value of 1.1 (each bucket is at most
    // 10% wider than the previous one), which will result in each power of
    // two divided into 8 buckets (e.g. there will be 8 buckets between 1
    // and 2, same as between 2 and 4, and 4 and 8, etc.).
    //
    // Details about the actually used factor: The factor is calculated as
    // 2^(2^-n), where n is an integer number between (and including) -4 and
    // 8. n is chosen so that the resulting factor is the largest that is
    // still smaller or equal to native_histogram_bucket_factor. Note that the
    // smallest possible factor is therefore approx. 1.00271 (i.e. 2^(2^-8)
    // ). If native_histogram_bucket_factor is greater than 1 but smaller than
    // 2^(2^-8), then the actually used factor is still 2^(2^-8) even though
    // it is larger than the provided native_histogram_bucket_factor.
    //
    // NOTE: Native Histograms are still an experimental feature. Their
    // behavior might still change without a major version
    // bump. Subsequently, all native_histogram... options here might still
    // change their behavior or name (or might completely disappear) without
    // a major version bump.
    pub native_histogram_bucket_factor: f64,
    // All observations with an absolute value of less or equal
    // native_histogram_zero_threshold are accumulated into a “zero” bucket.
    // For best results, this should be close to a bucket boundary. This is
    // usually the case if picking a power of two. If
    // native_histogram_zero_threshold is left at zero,
    // DEF_NATIVE_HISTOGRAM_ZERO_THRESHOLD is used as the threshold. To
    // configure a zero bucket with an actual threshold of zero (i.e. only
    // observations of precisely zero will go into the zero bucket), set
    // native_histogram_zero_threshold to the NATIVE_HISTOGRAM_ZERO_THRESHOLD_ZERO
    // constant (or any negative float value).
    pub native_histogram_zero_threshold: f64,

    // The next three fields define a strategy to limit the number of
    // populated sparse buckets. If native_histogram_max_bucket_number is left
    // at zero, the number of buckets is not limited. (Note that this might
    // lead to unbounded memory consumption if the values observed by the
    // Histogram are sufficiently wide-spread. In particular, this could be
    // used as a DoS attack vector. Where the observed values depend on
    // external inputs, it is highly recommended to set a
    // native_histogram_max_bucket_number.) Once the set
    // native_histogram_max_bucket_number is exceeded, the following strategy is
    // enacted:
    //  - First, if the last reset (or the creation) of the histogram is at
    //    least native_histogram_min_reset_duration ago, then the whole
    //    histogram is reset to its initial state (including regular
    //    buckets).
    //  - If less time has passed, or if native_histogram_min_reset_duration is
    //    zero, no reset is performed. Instead, the zero threshold is
    //    increased sufficiently to reduce the number of buckets to or below
    //    native_histogram_max_bucket_number, but not to more than
    //    native_histogram_max_zero_threshold. Thus, if
    //    native_histogram_max_zero_threshold is already at or below the current
    //    zero threshold, nothing happens at this step.
    //  - After that, if the number of buckets still exceeds
    //    native_histogram_max_bucket_number, the resolution of the histogram is
    //    reduced by doubling the width of the sparse buckets (up to a
    //    growth factor between one bucket to the next of 2^(2^4) = 65536,
    //    see above).
    //  - Any increased zero threshold or reduced resolution is reset back
    //    to their original values once native_histogram_min_reset_duration has
    //    passed (since the last reset or the creation of the histogram).
    pub native_histogram_max_bucket_number: u32,
    pub native_histogram_min_reset_duration: Duration,
    pub native_histogram_max_zero_threshold: f64,

    // native_histogram_max_exemplars limits the number of exemplars
    // that are kept in memory for each native histogram. If you leave it at
    // zero, a default value of 10 is used. If no exemplars should be kept specifically
    // for native histograms, set it to a negative value. (Scrapers can
    // still use the exemplars exposed for classic buckets, which are managed
    // independently.)
    pub native_histogram_max_exemplars: i32,
    // native_histogram_exemplar_ttl is only checked once
    // native_histogram_max_exemplars is exceeded. In that case, the
    // oldest exemplar is removed if it is older than native_histogram_exemplar_ttl.
    // Otherwise, the older exemplar in the pair of exemplars that are closest
    // together (on an exponential scale) is removed.
    // If native_histogram_exemplar_ttl is left at its zero value, a default value of
    // 5m is used. To always delete the oldest exemplar, set it to a negative value.
    pub native_histogram_exemplar_ttl: Duration,

    // now is for testing purposes, by default it's time::now.
    pub now: Option<fn() -> std::time::SystemTime>,

    // after_func is for testing purposes, by default it's time::after_func.
    pub after_func: Option<fn(Duration, fn()) -> std::thread::JoinHandle<()>>,
}

pub struct HistogramVecOpts {
    pub histogram_opts: HistogramOpts,

    // VariableLabels are used to partition the metric vector by the given set
    // of labels. Each label value will be constrained with the optional Constraint
    // function, if provided.
    pub variable_labels: ConstrainableLabels,
}

pub fn new_histogram(opts: HistogramOpts) -> Box<dyn Histogram> {
    let desc = Desc::new(
        build_fq_name(&opts.namespace, &opts.subsystem, &opts.name),
        &opts.help,
        vec![],
        opts.const_labels.clone(),
    );
    create_histogram(desc, opts)
}

fn create_histogram(desc: Desc, opts: HistogramOpts, label_values: Vec<String>) -> Box<dyn Histogram> {
    if desc.variable_labels.len() != label_values.len() {
        panic!("Inconsistent cardinality");
    }

    for name in &desc.variable_labels {
        if name == BUCKET_LABEL {
            panic!("{}", ERR_BUCKET_LABEL_NOT_ALLOWED);
        }
    }
    for lp in &desc.const_label_pairs {
        if lp.get_name() == BUCKET_LABEL {
            panic!("{}", ERR_BUCKET_LABEL_NOT_ALLOWED);
        }
    }

    let now = opts.now.unwrap_or_else(|| std::time::SystemTime::now);
    let after_func = opts.after_func.unwrap_or_else(|| std::thread::spawn);

    let mut h = HistogramImpl {
        desc,
        upper_bounds: opts.buckets.unwrap_or_else(|| DEF_BUCKETS.to_vec()),
        label_pairs: make_label_pairs(&desc, &label_values),
        native_histogram_max_buckets: opts.native_histogram_max_bucket_number,
        native_histogram_max_zero_threshold: opts.native_histogram_max_zero_threshold,
        native_histogram_min_reset_duration: opts.native_histogram_min_reset_duration,
        last_reset_time: now(),
        now,
        after_func,
        ..Default::default()
    };

    if h.upper_bounds.is_empty() && opts.native_histogram_bucket_factor <= 1.0 {
        h.upper_bounds = DEF_BUCKETS.to_vec();
    }
    if opts.native_histogram_bucket_factor <= 1.0 {
        h.native_histogram_schema = i32::MIN; // To mark that there are no sparse buckets.
    } else {
        h.native_histogram_zero_threshold = match opts.native_histogram_zero_threshold {
            t if t > 0.0 => t,
            0.0 => DEF_NATIVE_HISTOGRAM_ZERO_THRESHOLD,
            _ => 0.0,
        };
        h.native_histogram_schema = pick_schema(opts.native_histogram_bucket_factor);
        h.native_exemplars = make_native_exemplars(opts.native_histogram_exemplar_ttl, opts.native_histogram_max_exemplars);
    }

    for i in 0..h.upper_bounds.len() {
        if i < h.upper_bounds.len() - 1 {
            if h.upper_bounds[i] >= h.upper_bounds[i + 1] {
                panic!("histogram buckets must be in increasing order: {} >= {}", h.upper_bounds[i], h.upper_bounds[i + 1]);
            }
        } else if h.upper_bounds[i].is_infinite() {
            h.upper_bounds.pop();
        }
    }

    h.counts[0] = HistogramCounts::new(h.upper_bounds.len());
    h.counts[1] = HistogramCounts::new(h.upper_bounds.len());
    h.exemplars = vec![AtomicValue::default(); h.upper_bounds.len() + 1];

    h.init();
    Box::new(h)
}


use std::sync::atomic::{AtomicU64, AtomicI32, AtomicU32};
use std::collections::HashMap;
use std::sync::Mutex;

struct HistogramCounts {
    // Order in this struct matters for the alignment required by atomic
    // operations.

    // sum_bits contains the bits of the float64 representing the sum of all
    // observations.
    sum_bits: AtomicU64,
    count: AtomicU64,

    // native_histogram_zero_bucket counts all (positive and negative)
    // observations in the zero bucket (with an absolute value less or equal
    // the current threshold, see next field.
    native_histogram_zero_bucket: AtomicU64,
    // native_histogram_zero_threshold_bits is the bit pattern of the current
    // threshold for the zero bucket. It's initially equal to
    // native_histogram_zero_threshold but may change according to the bucket
    // count limitation strategy.
    native_histogram_zero_threshold_bits: AtomicU64,
    // native_histogram_schema may change over time according to the bucket
    // count limitation strategy and therefore has to be saved here.
    native_histogram_schema: AtomicI32,
    // Number of (positive and negative) sparse buckets.
    native_histogram_buckets_number: AtomicU32,

    // Regular buckets.
    buckets: Vec<AtomicU64>,

    // The sparse buckets for native histograms are implemented with a
    // Mutex<HashMap> for now. A dedicated data structure will likely be more
    // efficient. There are separate maps for negative and positive
    // observations. The map's value is an i64, counting observations in
    // that bucket. (Note that we don't use u64 as an i64 won't
    // overflow in practice, and working with signed numbers from the
    // beginning simplifies the handling of deltas.) The map's key is the
    // index of the bucket according to the used
    // native_histogram_schema. Index 0 is for an upper bound of 1.
    native_histogram_buckets_positive: Mutex<HashMap<i32, i64>>,
    native_histogram_buckets_negative: Mutex<HashMap<i32, i64>>,
}

impl HistogramCounts {
    fn new(bucket_count: usize) -> Self {
        HistogramCounts {
            sum_bits: AtomicU64::new(0),
            count: AtomicU64::new(0),
            native_histogram_zero_bucket: AtomicU64::new(0),
            native_histogram_zero_threshold_bits: AtomicU64::new(0),
            native_histogram_schema: AtomicI32::new(0),
            native_histogram_buckets_number: AtomicU32::new(0),
            buckets: (0..bucket_count).map(|_| AtomicU64::new(0)).collect(),
            native_histogram_buckets_positive: Mutex::new(HashMap::new()),
            native_histogram_buckets_negative: Mutex::new(HashMap::new()),
        }
    }
}

impl HistogramCounts {
    // observe manages the parts of observe that only affects
    // HistogramCounts. do_sparse is true if sparse buckets should be done,
    // too.
    fn observe(&self, v: f64, bucket: usize, do_sparse: bool) {
        if bucket < self.buckets.len() {
            self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
        }
        atomic_add_float(&self.sum_bits, v);
        if do_sparse && !v.is_nan() {
            let mut key;
            let schema = self.native_histogram_schema.load(Ordering::Relaxed);
            let zero_threshold = f64::from_bits(self.native_histogram_zero_threshold_bits.load(Ordering::Relaxed));
            let mut bucket_created = false;
            let is_inf;

            if v.is_infinite() {
                if v.is_sign_positive() {
                    v = f64::MAX;
                } else {
                    v = -f64::MAX;
                }
                is_inf = true;
            } else {
                is_inf = false;
            }

            let (frac, exp) = v.abs().frexp();
            if schema > 0 {
                let bounds = &NATIVE_HISTOGRAM_BOUNDS[schema as usize];
                key = bounds.binary_search_by(|&b| b.partial_cmp(&frac).unwrap()).unwrap_or_else(|x| x) + (exp - 1) * bounds.len();
            } else {
                key = exp;
                if frac == 0.5 {
                    key -= 1;
                }
                let offset = (1 << -schema) - 1;
                key = (key + offset) >> -schema;
            }
            if is_inf {
                key += 1;
            }

            if v > zero_threshold {
                bucket_created = add_to_bucket(&self.native_histogram_buckets_positive, key, 1);
            } else if v < -zero_threshold {
                bucket_created = add_to_bucket(&self.native_histogram_buckets_negative, key, 1);
            } else {
                self.native_histogram_zero_bucket.fetch_add(1, Ordering::Relaxed);
            }

            if bucket_created {
                self.native_histogram_buckets_number.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

use std::sync::{atomic::{AtomicU64, AtomicI32, AtomicU32, Ordering}, Mutex};
use std::time::{Duration, SystemTime};

struct Histogram {
    // count_and_hot_idx enables lock-free writes with use of atomic updates.
    // The most significant bit is the hot index [0 or 1] of the count field
    // below. Observe calls update the hot one. All remaining bits count the
    // number of Observe calls. Observe starts by incrementing this counter,
    // and finish by incrementing the count field in the respective
    // histogram_counts, as a marker for completion.
    //
    // Calls of the Write method (which are non-mutating reads from the
    // perspective of the histogram) swap the hot–cold under the write_mtx
    // lock. A cooldown is awaited (while locked) by comparing the number of
    // observations with the initiation count. Once they match, then the
    // last observation on the now cool one has completed. All cold fields must
    // be merged into the new hot before releasing write_mtx.
    //
    // Fields with atomic access first!
    count_and_hot_idx: AtomicU64,

    self_collector: SelfCollector,
    desc: Desc,

    // Only used in the Write method and for sparse bucket management.
    mtx: Mutex<()>,

    // Two counts, one is "hot" for lock-free observations, the other is
    // "cold" for writing out a dto.Metric. It has to be an array of
    // pointers to guarantee 64bit alignment of the histogram_counts.
    counts: [HistogramCounts; 2],

    upper_bounds: Vec<f64>,
    label_pairs: Vec<LabelPair>,
    exemplars: Vec<AtomicValue>, // One more than buckets (to include +Inf), each a *dto.Exemplar.
    native_histogram_schema: i32, // The initial schema. Set to i32::MIN if no sparse buckets are used.
    native_histogram_zero_threshold: f64, // The initial zero threshold.
    native_histogram_max_zero_threshold: f64,
    native_histogram_max_buckets: u32,
    native_histogram_min_reset_duration: Duration,
    // last_reset_time is protected by mtx. It is also used as created timestamp.
    last_reset_time: SystemTime,
    // reset_scheduled is protected by mtx. It is true if a reset is
    // scheduled for a later time (when native_histogram_min_reset_duration has
    // passed).
    reset_scheduled: bool,
    native_exemplars: NativeExemplars,

    // now is for testing purposes, by default it's SystemTime::now.
    now: fn() -> SystemTime,

    // after_func is for testing purposes, by default it's std::thread::spawn.
    after_func: fn(Duration, fn()) -> std::thread::JoinHandle<()>,
}

impl Histogram {
    pub fn desc(&self) -> &Desc {
        &self.desc
    }

    pub fn observe(&self, v: f64) {
        let bucket = self.find_bucket(v);
        self.observe_impl(v, bucket);
    }

    pub fn observe_with_exemplar(&self, v: f64, e: Labels) {
        let bucket = self.find_bucket(v);
        self.observe_impl(v, bucket);
        self.update_exemplar(v, bucket, e);
    }

    pub fn write(&self, out: &mut MetricFamily) -> Result<(), String> {
        let _lock = self.mtx.lock().unwrap();

        let n = self.count_and_hot_idx.fetch_add(1 << 63, Ordering::SeqCst);
        let count = n & ((1 << 63) - 1);
        let hot_counts = &self.counts[(n >> 63) as usize];
        let cold_counts = &self.counts[((^n) >> 63) as usize];

        wait_for_cooldown(count, cold_counts);

        let mut his = HistogramProto::default();
        his.sample_count = Some(count);
        his.sample_sum = Some(f64::from_bits(cold_counts.sum_bits.load(Ordering::Relaxed)));
        his.created_timestamp = Some(self.last_reset_time.into());

        out.histogram = Some(his);
        out.label = self.label_pairs.clone();

        let mut cum_count = 0;
        for (i, &upper_bound) in self.upper_bounds.iter().enumerate() {
            cum_count += cold_counts.buckets[i].load(Ordering::Relaxed);
            let mut bucket = BucketProto::default();
            bucket.cumulative_count = Some(cum_count);
            bucket.upper_bound = Some(upper_bound);
            if let Some(e) = self.exemplars[i].load(Ordering::Relaxed).as_ref() {
                bucket.exemplar = Some(e.clone());
            }
            out.histogram.as_mut().unwrap().bucket.push(bucket);
        }

        if let Some(e) = self.exemplars[self.upper_bounds.len()].load(Ordering::Relaxed).as_ref() {
            let mut bucket = BucketProto::default();
            bucket.cumulative_count = Some(count);
            bucket.upper_bound = Some(f64::INFINITY);
            bucket.exemplar = Some(e.clone());
            out.histogram.as_mut().unwrap().bucket.push(bucket);
        }

        if self.native_histogram_schema > i32::MIN {
            let zero_threshold = f64::from_bits(cold_counts.native_histogram_zero_threshold_bits.load(Ordering::Relaxed));
            let schema = cold_counts.native_histogram_schema.load(Ordering::Relaxed);
            let zero_bucket = cold_counts.native_histogram_zero_bucket.load(Ordering::Relaxed);

            out.histogram.as_mut().unwrap().zero_threshold = Some(zero_threshold);
            out.histogram.as_mut().unwrap().schema = Some(schema);
            out.histogram.as_mut().unwrap().zero_count = Some(zero_bucket);

            let (negative_span, negative_delta) = make_buckets(&cold_counts.native_histogram_buckets_negative);
            let (positive_span, positive_delta) = make_buckets(&cold_counts.native_histogram_buckets_positive);

            out.histogram.as_mut().unwrap().negative_span = negative_span;
            out.histogram.as_mut().unwrap().negative_delta = negative_delta;
            out.histogram.as_mut().unwrap().positive_span = positive_span;
            out.histogram.as_mut().unwrap().positive_delta = positive_delta;

            if zero_threshold == 0.0 && zero_bucket == 0 && positive_span.is_empty() && negative_span.is_empty() {
                out.histogram.as_mut().unwrap().positive_span.push(BucketSpanProto {
                    offset: Some(0),
                    length: Some(0),
                });
            }

            if self.native_exemplars.is_enabled() {
                let mut exemplars = self.native_exemplars.lock().unwrap();
                out.histogram.as_mut().unwrap().exemplars.append(&mut exemplars.exemplars);
            }
        }

        add_and_reset_counts(hot_counts, cold_counts);
        Ok(())
    }

    fn find_bucket(&self, v: f64) -> usize {
        let n = self.upper_bounds.len();
        if n == 0 {
            return 0;
        }

        if v <= self.upper_bounds[0] {
            return 0;
        }

        if v > self.upper_bounds[n - 1] {
            return n;
        }

        if n < 35 {
            for (i, &bound) in self.upper_bounds.iter().enumerate() {
                if v <= bound {
                    return i;
                }
            }
            return n;
        }

        self.upper_bounds.binary_search_by(|&bound| bound.partial_cmp(&v).unwrap()).unwrap_or_else(|x| x)
    }

    fn observe_impl(&self, v: f64, bucket: usize) {
        let do_sparse = self.native_histogram_schema > i32::MIN && !v.is_nan();
        let n = self.count_and_hot_idx.fetch_add(1, Ordering::SeqCst);
        let hot_counts = &self.counts[(n >> 63) as usize];
        hot_counts.observe(v, bucket, do_sparse);
        if do_sparse {
            self.limit_buckets(hot_counts, v, bucket);
        }
    }

    fn limit_buckets(&self, counts: &HistogramCounts, value: f64, bucket: usize) {
        if self.native_histogram_max_buckets == 0 {
            return;
        }
        if self.native_histogram_max_buckets >= counts.native_histogram_buckets_number.load(Ordering::Relaxed) {
            return;
        }

        let _lock = self.mtx.lock().unwrap();

        let n = self.count_and_hot_idx.load(Ordering::Relaxed);
        let hot_idx = (n >> 63) as usize;
        let cold_idx = ((^n) >> 63) as usize;
        let hot_counts = &self.counts[hot_idx];
        let cold_counts = &self.counts[cold_idx];

        if self.native_histogram_max_buckets >= hot_counts.native_histogram_buckets_number.load(Ordering::Relaxed) {
            return;
        }

        if self.maybe_reset(hot_counts, cold_counts, cold_idx, value, bucket) {
            return;
        }

        if self.native_histogram_min_reset_duration > Duration::new(0, 0) && !self.reset_scheduled {
            self.reset_scheduled = true;
            let now = (self.now)();
            let duration = self.native_histogram_min_reset_duration - now.duration_since(self.last_reset_time).unwrap();
            (self.after_func)(duration, self.reset);
        }

        if self.maybe_widen_zero_bucket(hot_counts, cold_counts) {
            return;
        }
        self.double_bucket_width(hot_counts, cold_counts);
    }

    // find_bucket returns the index of the bucket for the provided value, or
    // self.upper_bounds.len() for the +Inf bucket.
    fn find_bucket(&self, v: f64) -> usize {
        let n = self.upper_bounds.len();
        if n == 0 {
            return 0;
        }

        // Early exit: if v is less than or equal to the first upper bound, return 0
        if v <= self.upper_bounds[0] {
            return 0;
        }

        // Early exit: if v is greater than the last upper bound, return self.upper_bounds.len()
        if v > self.upper_bounds[n - 1] {
            return n;
        }

        // For small arrays, use simple linear search
        // "magic number" 35 is result of tests on couple different (AWS and baremetal) servers
        // see more details here: https://github.com/prometheus/client_golang/pull/1662
        if n < 35 {
            for (i, &bound) in self.upper_bounds.iter().enumerate() {
                if v <= bound {
                    return i;
                }
            }
            // If v is greater than all upper bounds, return self.upper_bounds.len()
            return n;
        }

        // For larger arrays, use stdlib's binary search
        self.upper_bounds.binary_search_by(|&bound| bound.partial_cmp(&v).unwrap()).unwrap_or_else(|x| x)
    }

    // observe is the implementation for Observe without the find_bucket part.
    fn observe_impl(&self, v: f64, bucket: usize) {
        // Do not add to sparse buckets for NaN observations.
        let do_sparse = self.native_histogram_schema > i32::MIN && !v.is_nan();
        // We increment self.count_and_hot_idx so that the counter in the lower
        // 63 bits gets incremented. At the same time, we get the new value
        // back, which we can use to find the currently-hot counts.
        let n = self.count_and_hot_idx.fetch_add(1, Ordering::SeqCst);
        let hot_counts = &self.counts[(n >> 63) as usize];
        hot_counts.observe(v, bucket, do_sparse);
        if do_sparse {
            self.limit_buckets(hot_counts, v, bucket);
        }
    }

    fn limit_buckets(&self, counts: &HistogramCounts, value: f64, bucket: usize) {
        if self.native_histogram_max_buckets == 0 {
            return; // No limit configured.
        }
        if self.native_histogram_max_buckets >= counts.native_histogram_buckets_number.load(Ordering::Relaxed) {
            return; // Bucket limit not exceeded yet.
        }

        let _lock = self.mtx.lock().unwrap();

        // The hot counts might have been swapped just before we acquired the
        // lock. Re-fetch the hot counts first...
        let n = self.count_and_hot_idx.load(Ordering::Relaxed);
        let hot_idx = (n >> 63) as usize;
        let cold_idx = ((^n) >> 63) as usize;
        let hot_counts = &self.counts[hot_idx];
        let cold_counts = &self.counts[cold_idx];
        // ...and then check again if we really have to reduce the bucket count.
        if self.native_histogram_max_buckets >= hot_counts.native_histogram_buckets_number.load(Ordering::Relaxed) {
            return; // Bucket limit not exceeded after all.
        }
        // Try the various strategies in order.
        if self.maybe_reset(hot_counts, cold_counts, cold_idx as u64, value, bucket) {
            return;
        }
        // One of the other strategies will happen. To undo what they will do as
        // soon as enough time has passed to satisfy
        // self.native_histogram_min_reset_duration, schedule a reset at the right time
        // if we haven't done so already.
        if self.native_histogram_min_reset_duration > Duration::new(0, 0) && !self.reset_scheduled {
            self.reset_scheduled = true;
            let now = (self.now)();
            let duration = self.native_histogram_min_reset_duration - now.duration_since(self.last_reset_time).unwrap();
            (self.after_func)(duration, self.reset);
        }

        if self.maybe_widen_zero_bucket(hot_counts, cold_counts) {
            return;
        }
        self.double_bucket_width(hot_counts, cold_counts);
    }

    fn maybe_reset(&self, hot: &HistogramCounts, cold: &HistogramCounts, cold_idx: u64, value: f64, bucket: usize) -> bool {
        if self.native_histogram_min_reset_duration == Duration::new(0, 0) || // No reset configured.
            self.reset_scheduled || // Do not interfere if a reset is already scheduled.
            (self.now)().duration_since(self.last_reset_time).unwrap() < self.native_histogram_min_reset_duration {
            return false;
        }
        // Completely reset cold_counts.
        self.reset_counts(cold);
        // Repeat the latest observation to not lose it completely.
        cold.observe(value, bucket, true);
        // Make cold_counts the new hot counts while resetting count_and_hot_idx.
        let n = self.count_and_hot_idx.swap((cold_idx << 63) + 1, Ordering::SeqCst);
        let count = n & ((1 << 63) - 1);
        wait_for_cooldown(count, hot);
        // Finally, reset the formerly hot counts, too.
        self.reset_counts(hot);
        self.last_reset_time = (self.now)();
        true
    }

    fn reset(&self) {
        let _lock = self.mtx.lock().unwrap();

        let n = self.count_and_hot_idx.load(Ordering::Relaxed);
        let hot_idx = (n >> 63) as usize;
        let cold_idx = ((^n) >> 63) as usize;
        let hot = &self.counts[hot_idx];
        let cold = &self.counts[cold_idx];
        // Completely reset cold_counts.
        self.reset_counts(cold);
        // Make cold_counts the new hot counts while resetting count_and_hot_idx.
        let n = self.count_and_hot_idx.swap((cold_idx << 63) as u64, Ordering::SeqCst);
        let count = n & ((1 << 63) - 1);
        wait_for_cooldown(count, hot);
        // Finally, reset the formerly hot counts, too.
        self.reset_counts(hot);
        self.last_reset_time = (self.now)();
        self.reset_scheduled = false;
    }

    fn maybe_widen_zero_bucket(&self, hot: &HistogramCounts, cold: &HistogramCounts) -> bool {
        let current_zero_threshold = f64::from_bits(hot.native_histogram_zero_threshold_bits.load(Ordering::Relaxed));
        if current_zero_threshold >= self.native_histogram_max_zero_threshold {
            return false;
        }
        // Find the key of the bucket closest to zero.
        let smallest_key = find_smallest_key(&hot.native_histogram_buckets_positive)
            .min(find_smallest_key(&hot.native_histogram_buckets_negative));
        if smallest_key == i32::MAX {
            return false;
        }
        let new_zero_threshold = get_le(smallest_key, hot.native_histogram_schema.load(Ordering::Relaxed));
        if new_zero_threshold > self.native_histogram_max_zero_threshold {
            return false; // New threshold would exceed the max threshold.
        }
        cold.native_histogram_zero_threshold_bits.store(f64::to_bits(new_zero_threshold), Ordering::Relaxed);
        // Remove applicable buckets.
        if cold.native_histogram_buckets_negative.lock().unwrap().remove(&smallest_key).is_some() {
            cold.native_histogram_buckets_number.fetch_sub(1, Ordering::Relaxed);
        }
        if cold.native_histogram_buckets_positive.lock().unwrap().remove(&smallest_key).is_some() {
            cold.native_histogram_buckets_number.fetch_sub(1, Ordering::Relaxed);
        }
        // Make cold counts the new hot counts.
        let n = self.count_and_hot_idx.fetch_add(1 << 63, Ordering::SeqCst);
        let count = n & ((1 << 63) - 1);
        // Swap the pointer names to represent the new roles and make
        // the rest less confusing.
        let (hot, cold) = (cold, hot);
        wait_for_cooldown(count, cold);
        // Add all the now cold counts to the new hot counts...
        add_and_reset_counts(hot, cold);
        // ...adjust the new zero threshold in the cold counts, too...
        cold.native_histogram_zero_threshold_bits.store(f64::to_bits(new_zero_threshold), Ordering::Relaxed);
        // ...and then merge the newly deleted buckets into the wider zero
        // bucket.
        let merge_and_delete_or_add_and_reset = |hot_buckets: &Mutex<HashMap<i32, i64>>, cold_buckets: &Mutex<HashMap<i32, i64>>| {
            let mut hot_buckets = hot_buckets.lock().unwrap();
            let mut cold_buckets = cold_buckets.lock().unwrap();
            for (&key, bucket) in cold_buckets.iter() {
                if key == smallest_key {
                    // Merge into hot zero bucket...
                    hot.native_histogram_zero_bucket.fetch_add(*bucket as u64, Ordering::Relaxed);
                    // ...and delete from cold counts.
                    cold_buckets.remove(&key);
                    cold.native_histogram_buckets_number.fetch_sub(1, Ordering::Relaxed);
                } else {
                    // Add to corresponding hot bucket...
                    if add_to_bucket(&mut hot_buckets, key, *bucket) {
                        hot.native_histogram_buckets_number.fetch_add(1, Ordering::Relaxed);
                    }
                    // ...and reset cold bucket.
                    *bucket = 0;
                }
            }
        };

        merge_and_delete_or_add_and_reset(&hot.native_histogram_buckets_positive, &cold.native_histogram_buckets_positive);
        merge_and_delete_or_add_and_reset(&hot.native_histogram_buckets_negative, &cold.native_histogram_buckets_negative);
        true
    }

    fn double_bucket_width(&self, hot: &HistogramCounts, cold: &HistogramCounts) {
        let cold_schema = cold.native_histogram_schema.load(Ordering::Relaxed);
        if cold_schema == -4 {
            return; // Already at lowest resolution.
        }
        let new_schema = cold_schema - 1;
        cold.native_histogram_schema.store(new_schema, Ordering::Relaxed);
        // Play it simple and just delete all cold buckets.
        cold.native_histogram_buckets_number.store(0, Ordering::Relaxed);
        cold.native_histogram_buckets_negative.lock().unwrap().clear();
        cold.native_histogram_buckets_positive.lock().unwrap().clear();
        // Make cold_counts the new hot counts.
        let n = self.count_and_hot_idx.fetch_add(1 << 63, Ordering::SeqCst);
        let count = n & ((1 << 63) - 1);
        // Swap the pointer names to represent the new roles and make
        // the rest less confusing.
        let (hot, cold) = (cold, hot);
        wait_for_cooldown(count, cold);
        // Add all the now cold counts to the new hot counts...
        add_and_reset_counts(hot, cold);
        // ...adjust the schema in the cold counts, too...
        cold.native_histogram_schema.store(new_schema, Ordering::Relaxed);
        // ...and then merge the cold buckets into the wider hot buckets.
        let merge = |hot_buckets: &Mutex<HashMap<i32, i64>>| {
            let mut hot_buckets = hot_buckets.lock().unwrap();
            for (&key, bucket) in cold.native_histogram_buckets_positive.lock().unwrap().iter() {
                let mut new_key = key;
                if new_key > 0 {
                    new_key += 1;
                }
                new_key /= 2;
                if add_to_bucket(&mut hot_buckets, new_key, *bucket) {
                    hot.native_histogram_buckets_number.fetch_add(1, Ordering::Relaxed);
                }
            }
        };

        merge(&hot.native_histogram_buckets_positive);
        merge(&hot.native_histogram_buckets_negative);
        // Play it simple again and just delete all cold buckets.
        cold.native_histogram_buckets_number.store(0, Ordering::Relaxed);
        cold.native_histogram_buckets_negative.lock().unwrap().clear();
        cold.native_histogram_buckets_positive.lock().unwrap().clear();
    }

    fn reset_counts(&self, counts: &HistogramCounts) {
        counts.sum_bits.store(0, Ordering::Relaxed);
        counts.count.store(0, Ordering::Relaxed);
        counts.native_histogram_zero_bucket.store(0, Ordering::Relaxed);
        counts.native_histogram_zero_threshold_bits.store(f64::to_bits(self.native_histogram_zero_threshold), Ordering::Relaxed);
        counts.native_histogram_schema.store(self.native_histogram_schema, Ordering::Relaxed);
        counts.native_histogram_buckets_number.store(0, Ordering::Relaxed);
        for bucket in &counts.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
        counts.native_histogram_buckets_negative.lock().unwrap().clear();
        counts.native_histogram_buckets_positive.lock().unwrap().clear();
    }

    fn update_exemplar(&self, v: f64, bucket: usize, l: Labels) {
        if l.is_empty() {
            return;
        }
        let e = new_exemplar(v, (self.now)(), l).expect("Invalid labels");
        self.exemplars[bucket].store(Some(e.clone()), Ordering::Relaxed);
        let do_sparse = self.native_histogram_schema > i32::MIN && !v.is_nan();
        if do_sparse {
            self.native_exemplars.add_exemplar(e);
        }
    }
}

struct HistogramVec {
    metric_vec: MetricVec,
}

impl HistogramVec {
    pub fn new(opts: HistogramOpts, label_names: Vec<String>) -> Self {
        let desc = Desc::new(
            &build_fq_name(&opts.namespace, &opts.subsystem, &opts.name),
            &opts.help,
            label_names.clone(),
            opts.const_labels.clone(),
        );
        HistogramVec {
            metric_vec: MetricVec::new(desc, move |lvs: Vec<String>| {
                create_histogram(desc.clone(), opts.clone(), lvs)
            }),
        }
    }

    pub fn get_metric_with_label_values(&self, lvs: &[&str]) -> Result<Box<dyn Observer>, String> {
        let metric = self.metric_vec.get_metric_with_label_values(lvs)?;
        Ok(metric.as_any().downcast_ref::<Box<dyn Observer>>().unwrap().clone())
    }

    pub fn get_metric_with(&self, labels: &HashMap<String, String>) -> Result<Box<dyn Observer>, String> {
        let metric = self.metric_vec.get_metric_with(labels)?;
        Ok(metric.as_any().downcast_ref::<Box<dyn Observer>>().unwrap().clone())
    }

    pub fn with_label_values(&self, lvs: &[&str]) -> Box<dyn Observer> {
        self.get_metric_with_label_values(lvs).unwrap()
    }

    pub fn with(&self, labels: &HashMap<String, String>) -> Box<dyn Observer> {
        self.get_metric_with(labels).unwrap()
    }

    pub fn curry_with(&self, labels: &HashMap<String, String>) -> Result<HistogramVec, String> {
        let vec = self.metric_vec.curry_with(labels)?;
        Ok(HistogramVec { metric_vec: vec })
    }

    pub fn must_curry_with(&self, labels: &HashMap<String, String>) -> HistogramVec {
        self.curry_with(labels).unwrap()
    }
}

struct ConstHistogram {
    desc: Desc,
    count: u64,
    sum: f64,
    buckets: HashMap<f64, u64>,
    label_pairs: Vec<LabelPair>,
    created_ts: Option<SystemTime>,
}

impl Metric for ConstHistogram {
    fn desc(&self) -> &Desc {
        &self.desc
    }

    fn write(&self, out: &mut MetricFamily) -> Result<(), String> {
        let mut his = HistogramProto::default();
        his.sample_count = Some(self.count);
        his.sample_sum = Some(self.sum);
        his.created_timestamp = self.created_ts.map(Into::into);

        let mut buckets: Vec<_> = self.buckets.iter().map(|(&upper_bound, &count)| {
            let mut bucket = BucketProto::default();
            bucket.cumulative_count = Some(count);
            bucket.upper_bound = Some(upper_bound);
            bucket
        }).collect();

        buckets.sort_by(|a, b| a.upper_bound.partial_cmp(&b.upper_bound).unwrap());
        his.bucket = buckets;

        out.histogram = Some(his);
        out.label = self.label_pairs.clone();

        Ok(())
    }
}

pub fn new_const_histogram(
    desc: Desc,
    count: u64,
    sum: f64,
    buckets: HashMap<f64, u64>,
    label_values: Vec<String>,
) -> Result<Box<dyn Metric>, String> {
    if let Some(err) = &desc.err {
        return Err(err.clone());
    }
    validate_label_values(&label_values, desc.variable_labels.len())?;
    Ok(Box::new(ConstHistogram {
        desc,
        count,
        sum,
        buckets,
        label_pairs: make_label_pairs(&desc, &label_values),
        created_ts: None,
    }))
}

pub fn must_new_const_histogram(
    desc: Desc,
    count: u64,
    sum: f64,
    buckets: HashMap<f64, u64>,
    label_values: Vec<String>,
) -> Box<dyn Metric> {
    new_const_histogram(desc, count, sum, buckets, label_values).unwrap()
}

pub fn new_const_histogram_with_created_timestamp(
    desc: Desc,
    count: u64,
    sum: f64,
    buckets: HashMap<f64, u64>,
    ct: SystemTime,
    label_values: Vec<String>,
) -> Result<Box<dyn Metric>, String> {
    if let Some(err) = &desc.err {
        return Err(err.clone());
    }
    validate_label_values(&label_values, desc.variable_labels.len())?;
    Ok(Box::new(ConstHistogram {
        desc,
        count,
        sum,
        buckets,
        label_pairs: make_label_pairs(&desc, &label_values),
        created_ts: Some(ct),
    }))
}

pub fn must_new_const_histogram_with_created_timestamp(
    desc: Desc,
    count: u64,
    sum: f64,
    buckets: HashMap<f64, u64>,
    ct: SystemTime,
    label_values: Vec<String>,
) -> Box<dyn Metric> {
    new_const_histogram_with_created_timestamp(desc, count, sum, buckets, ct, label_values).unwrap()
}

struct BuckSort(Vec<BucketProto>);

impl BuckSort {
    fn new(buckets: Vec<BucketProto>) -> Self {
        BuckSort(buckets)
    }
}

impl std::ops::Deref for BuckSort {
    type Target = Vec<BucketProto>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BuckSort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::cmp::PartialEq for BuckSort {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::cmp::Eq for BuckSort {}

impl std::cmp::PartialOrd for BuckSort {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for BuckSort {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl std::cmp::PartialOrd<BucketProto> for BuckSort {
    fn partial_cmp(&self, other: &BucketProto) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.upper_bound)
    }
}

impl std::cmp::Ord for BucketProto {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.upper_bound.partial_cmp(&other.upper_bound).unwrap()
    }
}

fn pick_schema(bucket_factor: f64) -> i32 {
    if bucket_factor <= 1.0 {
        panic!("bucket_factor is <= 1");
    }
    let floor = (bucket_factor.log2()).log2().floor();
    match floor {
        f if f <= -8.0 => NATIVE_HISTOGRAM_SCHEMA_MAXIMUM,
        f if f >= 4.0 => NATIVE_HISTOGRAM_SCHEMA_MINIMUM,
        _ => -floor as i32,
    }
}

fn make_buckets(buckets: &Mutex<HashMap<i32, i64>>) -> (Vec<BucketSpan>, Vec<i64>) {
    let mut ii: Vec<_> = buckets.lock().unwrap().keys().cloned().collect();
    ii.sort();

    if ii.is_empty() {
        return (vec![], vec![]);
    }

    let mut spans = vec![];
    let mut deltas = vec![];
    let mut prev_count = 0;
    let mut next_i = 0;

    let mut append_delta = |count: i64| {
        spans.last_mut().unwrap().length += 1;
        deltas.push(count - prev_count);
        prev_count = count;
    };

    for (n, &i) in ii.iter().enumerate() {
        let count = *buckets.lock().unwrap().get(&i).unwrap();
        let i_delta = i - next_i;
        if n == 0 || i_delta > 2 {
            spans.push(BucketSpan { offset: i_delta, length: 0 });
        } else {
            for _ in 0..i_delta {
                append_delta(0);
            }
        }
        append_delta(count);
        next_i = i + 1;
    }
    (spans, deltas)
}

fn add_to_bucket(buckets: &Mutex<HashMap<i32, i64>>, key: i32, increment: i64) -> bool {
    let mut buckets = buckets.lock().unwrap();
    if let Some(existing_bucket) = buckets.get_mut(&key) {
        *existing_bucket += increment;
        false
    } else {
        buckets.insert(key, increment);
        true
    }
}

fn add_and_reset(hot_buckets: &Mutex<HashMap<i32, i64>>, bucket_number: &AtomicU32) -> impl FnMut(&i32, &i64) -> bool {
    move |key, bucket| {
        if add_to_bucket(hot_buckets, *key, *bucket) {
            bucket_number.fetch_add(1, Ordering::Relaxed);
        }
        true
    }
}

fn delete_sync_map(m: &Mutex<HashMap<i32, i64>>) {
    m.lock().unwrap().clear();
}

fn find_smallest_key(m: &Mutex<HashMap<i32, i64>>) -> i32 {
    *m.lock().unwrap().keys().min().unwrap_or(&i32::MAX)
}

fn get_le(key: i32, schema: i32) -> f64 {
    if schema < 0 {
        let exp = key << -schema;
        if exp == 1024 {
            return f64::MAX;
        }
        return f64::from_bits(1 << exp);
    }

    let frac_idx = key & ((1 << schema) - 1);
    let frac = NATIVE_HISTOGRAM_BOUNDS[schema as usize][frac_idx as usize];
    let exp = (key >> schema) + 1;
    if frac == 0.5 && exp == 1025 {
        return f64::MAX;
    }
    f64::from_bits(frac.to_bits() << exp)
}

fn wait_for_cooldown(count: u64, counts: &HistogramCounts) {
    while count != counts.count.load(Ordering::Relaxed) {
        std::thread::yield_now();
    }
}

fn atomic_add_float(bits: &AtomicU64, v: f64) {
    let mut old_bits = bits.load(Ordering::Relaxed);
    loop {
        let old_val = f64::from_bits(old_bits);
        let new_val = old_val + v;
        let new_bits = new_val.to_bits();
        match bits.compare_exchange_weak(old_bits, new_bits, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(x) => old_bits = x,
        }
    }
}

fn atomic_dec_u32(p: &AtomicU32) {
    p.fetch_sub(1, Ordering::Relaxed);
}

fn add_and_reset_counts(hot: &HistogramCounts, cold: &HistogramCounts) {
    hot.count.fetch_add(cold.count.load(Ordering::Relaxed), Ordering::Relaxed);
    cold.count.store(0, Ordering::Relaxed);
    let cold_sum = f64::from_bits(cold.sum_bits.load(Ordering::Relaxed));
    atomic_add_float(&hot.sum_bits, cold_sum);
    cold.sum_bits.store(0, Ordering::Relaxed);
    for (hot_bucket, cold_bucket) in hot.buckets.iter().zip(&cold.buckets) {
        hot_bucket.fetch_add(cold_bucket.load(Ordering::Relaxed), Ordering::Relaxed);
        cold_bucket.store(0, Ordering::Relaxed);
    }
    hot.native_histogram_zero_bucket.fetch_add(cold.native_histogram_zero_bucket.load(Ordering::Relaxed), Ordering::Relaxed);
    cold.native_histogram_zero_bucket.store(0, Ordering::Relaxed);
}

struct NativeExemplars {
    ttl: Duration,
    exemplars: Vec<Exemplar>,
}

impl NativeExemplars {
    fn is_enabled(&self) -> bool {
        self.ttl != Duration::from_secs(-1)
    }

    fn new(ttl: Duration, max_count: i32) -> Self {
        let ttl = if ttl == Duration::new(0, 0) {
            Duration::from_secs(300)
        } else {
            ttl
        };

        let max_count = if max_count == 0 {
            10
        } else if max_count < 0 {
            0
        } else {
            max_count
        };

        NativeExemplars {
            ttl,
            exemplars: Vec::with_capacity(max_count as usize),
        }
    }

    fn add_exemplar(&mut self, e: Exemplar) {
        if !self.is_enabled() {
            return;
        }

        // When the number of exemplars has not yet exceeded or
        // is equal to cap(self.exemplars), then
        // insert the new exemplar directly.
        if self.exemplars.len() < self.exemplars.capacity() {
            let n_idx = self.exemplars.iter().position(|ex| e.value < ex.value).unwrap_or(self.exemplars.len());
            self.exemplars.insert(n_idx, e);
            return;
        }

        if self.exemplars.len() == 1 {
            // When the number of exemplars is 1, then
            // replace the existing exemplar with the new exemplar.
            self.exemplars[0] = e;
            return;
        }

        // From this point on, the number of exemplars is greater than 1.
        let mut ot = SystemTime::UNIX_EPOCH;
        let mut ot_idx = -1;
        let mut md = -1.0;
        let mut n_idx = -1;
        let mut r_idx = -1;
        let mut c_log = 0.0;
        let mut p_log = 0.0;

        for (i, exemplar) in self.exemplars.iter().enumerate() {
            // Find the exemplar with the oldest timestamp.
            if ot_idx == -1 || exemplar.timestamp < ot {
                ot = exemplar.timestamp;
                ot_idx = i as i32;
            }

            // Find the index at which to insert new the exemplar.
            if n_idx == -1 && e.value <= exemplar.value {
                n_idx = i as i32;
            }

            // Find the two closest exemplars and pick the one the with older timestamp.
            p_log = c_log;
            c_log = exemplar.value.ln();
            if i == 0 {
                continue;
            }
            let diff = (c_log - p_log).abs();
            if md == -1.0 || diff < md {
                md = diff;
                if self.exemplars[i].timestamp < self.exemplars[i - 1].timestamp {
                    r_idx = i as i32;
                } else {
                    r_idx = (i - 1) as i32;
                }
            }
        }

        if n_idx == -1 {
            n_idx = self.exemplars.len() as i32;
        }

        if ot_idx != -1 && e.timestamp.duration_since(ot).unwrap() > self.ttl {
            r_idx = ot_idx;
        } else {
            let e_log = e.value.ln();
            if n_idx > 0 {
                let diff = (e_log - self.exemplars[(n_idx - 1) as usize].value.ln()).abs();
                if diff < md {
                    r_idx = n_idx - 1;
                }
            }
            if n_idx < self.exemplars.len() as i32 {
                let diff = (self.exemplars[n_idx as usize].value.ln() - e_log).abs();
                if diff < md {
                    r_idx = n_idx;
                }
            }
        }

        match r_idx.cmp(&n_idx) {
            std::cmp::Ordering::Equal => self.exemplars[r_idx as usize] = e,
            std::cmp::Ordering::Less => {
                self.exemplars.remove(r_idx as usize);
                self.exemplars.insert(n_idx as usize - 1, e);
            }
            std::cmp::Ordering::Greater => {
                self.exemplars.remove(r_idx as usize);
                self.exemplars.insert(n_idx as usize, e);
            }
        }
    }
}

struct ConstNativeHistogram {
    desc: Desc,
    histogram: HistogramProto,
    label_pairs: Vec<LabelPair>,
}

impl Metric for ConstNativeHistogram {
    fn desc(&self) -> &Desc {
        &self.desc
    }

    fn write(&self, out: &mut MetricFamily) -> Result<(), String> {
        out.histogram = Some(self.histogram.clone());
        out.label = self.label_pairs.clone();
        Ok(())
    }
}

fn validate_count(sum: f64, count: u64, negative_buckets: &HashMap<i32, i64>, positive_buckets: &HashMap<i32, i64>, zero_bucket: u64) -> Result<(), String> {
    let bucket_population_sum: i64 = negative_buckets.values().sum::<i64>() + positive_buckets.values().sum::<i64>() + zero_bucket as i64;

    if (sum.is_nan() && bucket_population_sum > count as i64) || (!sum.is_nan() && bucket_population_sum != count as i64) {
        return Err("the sum of all bucket populations exceeds the count of observations".to_string());
    }
    Ok(())
}

fn new_const_native_histogram(
    desc: Desc,
    count: u64,
    sum: f64,
    positive_buckets: HashMap<i32, i64>,
    negative_buckets: HashMap<i32, i64>,
    zero_bucket: u64,
    schema: i32,
    zero_threshold: f64,
    created_timestamp: SystemTime,
    label_values: Vec<String>,
) -> Result<Box<dyn Metric>, String> {
    if let Some(err) = &desc.err {
        return Err(err.clone());
    }
    validate_label_values(&label_values, desc.variable_labels.len())?;
    if schema > NATIVE_HISTOGRAM_SCHEMA_MAXIMUM || schema < NATIVE_HISTOGRAM_SCHEMA_MINIMUM {
        return Err("invalid native histogram schema".to_string());
    }
    validate_count(sum, count, &negative_buckets, &positive_buckets, zero_bucket)?;

    let (negative_span, negative_delta) = make_buckets_from_map(&negative_buckets);
    let (positive_span, positive_delta) = make_buckets_from_map(&positive_buckets);

    let mut histogram = HistogramProto {
        created_timestamp: Some(created_timestamp.into()),
        schema: Some(schema),
        zero_threshold: Some(zero_threshold),
        sample_count: Some(count),
        sample_sum: Some(sum),
        negative_span,
        negative_delta,
        positive_span,
        positive_delta,
        zero_count: Some(zero_bucket),
        ..Default::default()
    };

    if histogram.zero_threshold == Some(0.0) && histogram.zero_count == Some(0) && histogram.positive_span.is_empty() && histogram.negative_span.is_empty() {
        histogram.positive_span.push(BucketSpan {
            offset: 0,
            length: 0,
        });
    }

    Ok(Box::new(ConstNativeHistogram {
        desc,
        histogram,
        label_pairs: make_label_pairs(&desc, &label_values),
    }))
}

fn must_new_const_native_histogram(
    desc: Desc,
    count: u64,
    sum: f64,
    positive_buckets: HashMap<i32, i64>,
    negative_buckets: HashMap<i32, i64>,
    zero_bucket: u64,
    schema: i32,
    zero_threshold: f64,
    created_timestamp: SystemTime,
    label_values: Vec<String>,
) -> Box<dyn Metric> {
    new_const_native_histogram(
        desc,
        count,
        sum,
        positive_buckets,
        negative_buckets,
        zero_bucket,
        schema,
        zero_threshold,
        created_timestamp,
        label_values,
    )
    .unwrap()
}

fn make_buckets_from_map(buckets: &HashMap<i32, i64>) -> (Vec<BucketSpan>, Vec<i64>) {
    if buckets.is_empty() {
        return (vec![], vec![]);
    }

    let mut ii: Vec<_> = buckets.keys().cloned().collect();
    ii.sort();

    let mut spans = vec![];
    let mut deltas = vec![];
    let mut prev_count = 0;
    let mut next_i = 0;

    let mut append_delta = |count: i64| {
        spans.last_mut().unwrap().length += 1;
        deltas.push(count - prev_count);
        prev_count = count;
    };

    for (n, &i) in ii.iter().enumerate() {
        let count = buckets[&i];
        let i_delta = i - next_i;
        if n == 0 || i_delta > 2 {
            spans.push(BucketSpan {
                offset: i_delta,
                length: 0,
            });
        } else {
            for _ in 0..i_delta {
                append_delta(0);
            }
        }
        append_delta(count);
        next_i = i + 1;
    }
    (spans, deltas)
}