fn min_int(a: i32, b: i32) -> i32 {
    if a < b {
        return a;
    }
    b
}

fn max_int(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    }
    b
}

fn calculate_ratio(matches: i32, length: i32) -> f64 {
    if length > 0 {
        return 2.0 * matches as f64 / length as f64;
    }
    1.0
}

#[derive(Debug)]
struct Match {
    a: usize,
    b: usize,
    size: usize,
}

#[derive(Debug)]
struct OpCode {
    tag: u8,
    i1: usize,
    i2: usize,
    j1: usize,
    j2: usize,
}

#[derive(Debug)]
struct SequenceMatcher<'a> {
    a: Vec<&'a str>,
    b: Vec<&'a str>,
    b2j: HashMap<&'a str, Vec<usize>>,
    is_junk: Option<Box<dyn Fn(&str) -> bool + 'a>>,
    auto_junk: bool,
    b_junk: HashSet<&'a str>,
    matching_blocks: Vec<Match>,
    full_b_count: HashMap<&'a str, usize>,
    b_popular: HashSet<&'a str>,
    op_codes: Vec<OpCode>,
}

impl<'a> SequenceMatcher<'a> {
    fn new(a: Vec<&'a str>, b: Vec<&'a str>) -> Self {
        let mut matcher = SequenceMatcher {
            a: Vec::new(),
            b: Vec::new(),
            b2j: HashMap::new(),
            is_junk: None,
            auto_junk: true,
            b_junk: HashSet::new(),
            matching_blocks: Vec::new(),
            full_b_count: HashMap::new(),
            b_popular: HashSet::new(),
            op_codes: Vec::new(),
        };
        matcher.set_seqs(a, b);
        matcher
    }

    fn new_with_junk(
        a: Vec<&'a str>,
        b: Vec<&'a str>,
        auto_junk: bool,
        is_junk: Option<Box<dyn Fn(&str) -> bool + 'a>>,
    ) -> Self {
        let mut matcher = SequenceMatcher {
            a: Vec::new(),
            b: Vec::new(),
            b2j: HashMap::new(),
            is_junk,
            auto_junk,
            b_junk: HashSet::new(),
            matching_blocks: Vec::new(),
            full_b_count: HashMap::new(),
            b_popular: HashSet::new(),
            op_codes: Vec::new(),
        };
        matcher.set_seqs(a, b);
        matcher
    }

    fn set_seqs(&mut self, a: Vec<&'a str>, b: Vec<&'a str>) {
        self.set_seq1(a);
        self.set_seq2(b);
    }

    fn set_seq1(&mut self, a: Vec<&'a str>) {
        if self.a.as_ptr() == a.as_ptr() {
            return;
        }
        self.a = a;
        self.matching_blocks = None;
        self.op_codes = None;
    }

    fn set_seq2(&mut self, b: Vec<&'a str>) {
        if self.b.as_ptr() == b.as_ptr() {
            return;
        }
        self.b = b;
        self.matching_blocks = None;
        self.op_codes = None;
        self.full_b_count = None;
        self.chain_b();
    }

    fn chain_b(&mut self) {
        // Populate line -> index mapping
        let mut b2j: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, &s) in self.b.iter().enumerate() {
            b2j.entry(s).or_insert_with(Vec::new).push(i);
        }

        // Purge junk elements
        self.b_junk = HashSet::new();
        if let Some(ref is_junk) = self.is_junk {
            let junk = &mut self.b_junk;
            for &s in b2j.keys() {
                if is_junk(s) {
                    junk.insert(s);
                }
            }
            for &s in junk {
                b2j.remove(s);
            }
        }

        // Purge remaining popular elements
        let mut popular = HashSet::new();
        let n = self.b.len();
        if self.auto_junk && n >= 200 {
            let ntest = n / 100 + 1;
            for (&s, indices) in &b2j {
                if indices.len() > ntest {
                    popular.insert(s);
                }
            }
            for &s in &popular {
                b2j.remove(s);
            }
        }
        self.b_popular = popular;
        self.b2j = b2j;
    }

    fn is_b_junk(&self, s: &str) -> bool {
        self.b_junk.contains(s)
    }

    fn find_longest_match(&self, alo: usize, ahi: usize, blo: usize, bhi: usize) -> Match {
        let mut besti = alo;
        let mut bestj = blo;
        let mut bestsize = 0;

        let mut j2len: HashMap<usize, usize> = HashMap::new();
        for i in alo..ahi {
            let mut newj2len: HashMap<usize, usize> = HashMap::new();
            if let Some(indices) = self.b2j.get(self.a[i]) {
                for &j in indices {
                    if j < blo {
                        continue;
                    }
                    if j >= bhi {
                        break;
                    }
                    let k = j2len.get(&(j - 1)).unwrap_or(&0) + 1;
                    newj2len.insert(j, k);
                    if k > bestsize {
                        besti = i + 1 - k;
                        bestj = j + 1 - k;
                        bestsize = k;
                    }
                }
            }
            j2len = newj2len;
        }

        while besti > alo && bestj > blo && !self.is_b_junk(self.b[bestj - 1]) && self.a[besti - 1] == self.b[bestj - 1] {
            besti -= 1;
            bestj -= 1;
            bestsize += 1;
        }
        while besti + bestsize < ahi && bestj + bestsize < bhi && !self.is_b_junk(self.b[bestj + bestsize]) && self.a[besti + bestsize] == self.b[bestj + bestsize] {
            bestsize += 1;
        }

        while besti > alo && bestj > blo && self.is_b_junk(self.b[bestj - 1]) && self.a[besti - 1] == self.b[bestj - 1] {
            besti -= 1;
            bestj -= 1;
            bestsize += 1;
        }
        while besti + bestsize < ahi && bestj + bestsize < bhi && self.is_b_junk(self.b[bestj + bestsize]) && self.a[besti + bestsize] == self.b[bestj + bestsize] {
            bestsize += 1;
        }

        Match {
            a: besti,
            b: bestj,
            size: bestsize,
        }
    }

    fn get_matching_blocks(&mut self) -> Vec<Match> {
        if let Some(ref matching_blocks) = self.matching_blocks {
            return matching_blocks.clone();
        }

        fn match_blocks(
            m: &SequenceMatcher,
            alo: usize,
            ahi: usize,
            blo: usize,
            bhi: usize,
            mut matched: Vec<Match>,
        ) -> Vec<Match> {
            let match_ = m.find_longest_match(alo, ahi, blo, bhi);
            let (i, j, k) = (match_.a, match_.b, match_.size);
            if k > 0 {
                if alo < i && blo < j {
                    matched = match_blocks(m, alo, i, blo, j, matched);
                }
                matched.push(match_);
                if i + k < ahi && j + k < bhi {
                    matched = match_blocks(m, i + k, ahi, j + k, bhi, matched);
                }
            }
            matched
        }

        let matched = match_blocks(self, 0, self.a.len(), 0, self.b.len(), Vec::new());

        // It's possible that we have adjacent equal blocks in the matching_blocks list now.
        let mut non_adjacent = Vec::new();
        let (mut i1, mut j1, mut k1) = (0, 0, 0);
        for b in matched {
            // Is this block adjacent to i1, j1, k1?
            let (i2, j2, k2) = (b.a, b.b, b.size);
            if i1 + k1 == i2 && j1 + k1 == j2 {
                // Yes, so collapse them -- this just increases the length of
                // the first block by the length of the second, and the first
                // block so lengthened remains the block to compare against.
                k1 += k2;
            } else {
                // Not adjacent. Remember the first block (k1==0 means it's
                // the dummy we started with), and make the second block the
                // new block to compare against.
                if k1 > 0 {
                    non_adjacent.push(Match { a: i1, b: j1, size: k1 });
                }
                i1 = i2;
                j1 = j2;
                k1 = k2;
            }
        }
        if k1 > 0 {
            non_adjacent.push(Match { a: i1, b: j1, size: k1 });
        }

        non_adjacent.push(Match {
            a: self.a.len(),
            b: self.b.len(),
            size: 0,
        });
        self.matching_blocks = Some(non_adjacent.clone());
        non_adjacent
    }

    fn get_op_codes(&mut self) -> Vec<OpCode> {
        if let Some(ref op_codes) = self.op_codes {
            return op_codes.clone();
        }

        let mut i = 0;
        let mut j = 0;
        let matching = self.get_matching_blocks();
        let mut op_codes = Vec::with_capacity(matching.len());

        for m in matching {
            // invariant: we've pumped out correct diffs to change
            // a[:i] into b[:j], and the next matching block is
            // a[ai:ai+size] == b[bj:bj+size]. So we need to pump
            // out a diff to change a[i:ai] into b[j:bj], pump out
            // the matching block, and move (i,j) beyond the match
            let (ai, bj, size) = (m.a, m.b, m.size);
            let tag = if i < ai && j < bj {
                b'r'
            } else if i < ai {
                b'd'
            } else if j < bj {
                b'i'
            } else {
                0
            };
            if tag > 0 {
                op_codes.push(OpCode {
                    tag,
                    i1: i,
                    i2: ai,
                    j1: j,
                    j2: bj,
                });
            }
            i = ai + size;
            j = bj + size;
            // the list of matching blocks is terminated by a
            // sentinel with size 0
            if size > 0 {
                op_codes.push(OpCode {
                    tag: b'e',
                    i1: ai,
                    i2: i,
                    j1: bj,
                    j2: j,
                });
            }
        }

        self.op_codes = Some(op_codes.clone());
        op_codes
    }

    fn get_grouped_op_codes(&mut self, mut n: usize) -> Vec<Vec<OpCode>> {
        if n < 0 {
            n = 3;
        }
        let mut codes = self.get_op_codes();
        if codes.is_empty() {
            codes = vec![OpCode {
                tag: b'e',
                i1: 0,
                i2: 1,
                j1: 0,
                j2: 1,
            }];
        }

        // Fixup leading and trailing groups if they show no changes.
        if codes[0].tag == b'e' {
            let c = &codes[0];
            let (i1, i2, j1, j2) = (c.i1, c.i2, c.j1, c.j2);
            codes[0] = OpCode {
                tag: c.tag,
                i1: max_int(i1, i2.saturating_sub(n)),
                i2,
                j1: max_int(j1, j2.saturating_sub(n)),
                j2,
            };
        }
        if codes[codes.len() - 1].tag == b'e' {
            let c = &codes[codes.len() - 1];
            let (i1, i2, j1, j2) = (c.i1, c.i2, c.j1, c.j2);
            codes[codes.len() - 1] = OpCode {
                tag: c.tag,
                i1,
                i2: min_int(i2, i1 + n),
                j1,
                j2: min_int(j2, j1 + n),
            };
        }

        let nn = n + n;
        let mut groups = Vec::new();
        let mut group = Vec::new();
        for c in codes {
            let (mut i1, i2, mut j1, j2) = (c.i1, c.i2, c.j1, c.j2);
            // End the current group and start a new one whenever
            // there is a large range with no changes.
            if c.tag == b'e' && i2 - i1 > nn {
                group.push(OpCode {
                    tag: c.tag,
                    i1,
                    i2: min_int(i2, i1 + n),
                    j1,
                    j2: min_int(j2, j1 + n),
                });
                groups.push(group);
                group = Vec::new();
                i1 = max_int(i1, i2.saturating_sub(n));
                j1 = max_int(j1, j2.saturating_sub(n));
            }
            group.push(OpCode {
                tag: c.tag,
                i1,
                i2,
                j1,
                j2,
            });
        }
        if !group.is_empty() && !(group.len() == 1 && group[0].tag == b'e') {
            groups.push(group);
        }
        groups
    }

    fn ratio(&mut self) -> f64 {
        let mut matches = 0;
        for m in self.get_matching_blocks() {
            matches += m.size;
        }
        calculate_ratio(matches, self.a.len() + self.b.len())
    }

    fn quick_ratio(&mut self) -> f64 {
        // Viewing a and b as multisets, set matches to the cardinality
        // of their intersection; this counts the number of matches
        // without regard to order, so is clearly an upper bound
        if self.full_b_count.is_none() {
            let mut full_b_count = HashMap::new();
            for &s in &self.b {
                *full_b_count.entry(s).or_insert(0) += 1;
            }
            self.full_b_count = Some(full_b_count);
        }

        // avail[x] is the number of times x appears in 'b' less the
        // number of times we've seen it in 'a' so far ... kinda
        let mut avail = HashMap::new();
        let mut matches = 0;
        for &s in &self.a {
            let n = avail.get(&s).cloned().unwrap_or_else(|| {
                self.full_b_count.as_ref().unwrap().get(&s).cloned().unwrap_or(0)
            });
            avail.insert(s, n - 1);
            if n > 0 {
                matches += 1;
            }
        }
        calculate_ratio(matches, self.a.len() + self.b.len())
    }

    fn real_quick_ratio(&self) -> f64 {
        let la = self.a.len();
        let lb = self.b.len();
        calculate_ratio(min_int(la, lb), la + lb)
    }
}

fn max_int(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

fn min_int(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

fn format_range_unified(start: usize, stop: usize) -> String {
    // Per the diff spec at http://www.unix.org/single_unix_specification/
    let mut beginning = start + 1; // lines start numbering with one
    let length = stop - start;
    if length == 1 {
        return beginning.to_string();
    }
    if length == 0 {
        beginning -= 1; // empty ranges begin at line just before the range
    }
    format!("{},{}", beginning, length)
}

struct UnifiedDiff<'a> {
    a: Vec<&'a str>,       // First sequence lines
    from_file: String,     // First file name
    from_date: String,     // First file time
    b: Vec<&'a str>,       // Second sequence lines
    to_file: String,       // Second file name
    to_date: String,       // Second file time
    eol: String,           // Headers end of line, defaults to LF
    context: usize,        // Number of context lines
}

impl<'a> SequenceMatcher<'a> {
    fn write_unified_diff<W: Write>(&mut self, writer: W, mut diff: UnifiedDiff) -> io::Result<()> {
        let mut buf = BufWriter::new(writer);
        let wf = |buf: &mut BufWriter<W>, format: &str, args: &[&dyn std::fmt::Display]| -> io::Result<()> {
            write!(buf, format, args)?;
            Ok(())
        };
        let ws = |buf: &mut BufWriter<W>, s: &str| -> io::Result<()> {
            buf.write_all(s.as_bytes())?;
            Ok(())
        };

        if diff.eol.is_empty() {
            diff.eol = "\n".to_string();
        }

        let mut started = false;
        let mut matcher = SequenceMatcher::new(diff.a.clone(), diff.b.clone());
        for g in matcher.get_grouped_op_codes(diff.context) {
            if !started {
                started = true;
                let from_date = if !diff.from_date.is_empty() {
                    format!("\t{}", diff.from_date)
                } else {
                    "".to_string()
                };
                let to_date = if !diff.to_date.is_empty() {
                    format!("\t{}", diff.to_date)
                } else {
                    "".to_string()
                };
                if !diff.from_file.is_empty() || !diff.to_file.is_empty() {
                    wf(&mut buf, &format!("--- {}{}{}", diff.from_file, from_date, diff.eol), &[])?;
                    wf(&mut buf, &format!("+++ {}{}{}", diff.to_file, to_date, diff.eol), &[])?;
                }
            }
            let first = &g[0];
            let last = &g[g.len() - 1];
            let range1 = format_range_unified(first.i1, last.i2);
            let range2 = format_range_unified(first.j1, last.j2);
            wf(&mut buf, &format!("@@ -{} +{} @@{}", range1, range2, diff.eol), &[])?;
            for c in g {
                let (i1, i2, j1, j2) = (c.i1, c.i2, c.j1, c.j2);
                if c.tag == b'e' {
                    for line in &diff.a[i1..i2] {
                        ws(&mut buf, &format!(" {}", line))?;
                    }
                    continue;
                }
                if c.tag == b'r' || c.tag == b'd' {
                    for line in &diff.a[i1..i2] {
                        ws(&mut buf, &format!("-{}", line))?;
                    }
                }
                if c.tag == b'r' || c.tag == b'i' {
                    for line in &diff.b[j1..j2] {
                        ws(&mut buf, &format!("+{}", line))?;
                    }
                }
            }
        }
        buf.flush()?;
        Ok(())
    }

    fn get_unified_diff_string(&mut self, diff: UnifiedDiff) -> Result<String> {
        let mut buffer = Vec::new();
        self.write_unified_diff(&mut buffer, diff)?;
        Ok(String::from_utf8(buffer).expect("Invalid UTF-8 sequence"))
    }
}

fn split_lines(s: &str) -> Vec<String> {
    let mut lines: Vec<String> = s.split_inclusive('\n').map(String::from).collect();
    if !s.ends_with('\n') {
        if let Some(last) = lines.last_mut() {
            last.push('\n');
        }
    }
    lines
}